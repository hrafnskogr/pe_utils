use crate::mem_utils::*;
use std::fmt;

/* TODO:
 * Name formatting:
 *  fn that states addr of when it is actually an offset
 * Features:
 *  provide FNs to get mem addr of a PE section, and FNs to get offset relative to base addr
 */



#[derive(Debug)]
pub struct PEImage
{
    pub base_addr: usize,
    pub name: String,
    optional_header_offset: u32,
    export_directory_offset: u32,
    export_directory_addr: usize,
}

impl PEImage
{
    pub unsafe fn new(addr: usize) -> PEImage
    {
        let mut pe = PEImage {  base_addr: addr,
                                name: "uninit_dll".to_string(),
                                optional_header_offset: 0,
                                export_directory_offset: 0,
                                export_directory_addr: 0,
                            };
        pe.init();
        pe
    }

    pub unsafe fn init(&mut self)
    {
        // Read the file header offset located at offset 0x3c
        // Add 0x4 to the read offset to skip the PE Signature
        let file_header: u32 = (*((self.base_addr + 0x3c) as *const u32)) + 0x4;
        // The offset for the optional header is at 0x14
        self.optional_header_offset = file_header + 0x14;
        // Compute the offset where the offset to the export directory is stored, and retrieve it
        self.export_directory_offset = *((self.base_addr
                                            + self.optional_header_offset as usize
                                            + 0x70) as *const u32);

        // Compute a final absolute address to the export directory
        self.export_directory_addr = self.base_addr + self.export_directory_offset as usize;

        self.name = self.get_name();
    }

    pub unsafe fn get_export_directory_ptr(&self) -> *const usize
    {
        (self.base_addr + self.export_directory_offset as usize) as *const usize
    }

    unsafe fn get_name(&self) -> String
    {
        let name_offset = *((self.base_addr + self.export_directory_offset as usize + 0xC) as *const u32);
        let name_addr = self.base_addr + name_offset as usize;

        let (name, _) = read_until_null(name_addr as usize);
        let name = String::from_utf8(name).unwrap();

        name
    }

    pub unsafe fn number_of_func(&self) -> u32
    {
        *((self.export_directory_addr + 0x14) as *const u32) 
    }

    pub unsafe fn number_of_names(&self) -> u32
    {
        *((self.export_directory_addr + 0x18) as *const u32) 
    }

    // TODO: fix these names, this is actually an offset
    pub unsafe fn addr_of_func(&self) -> usize
    {
        *((self.export_directory_addr + 0x1c) as *const u32) as usize
    }
    
    pub unsafe fn addr_of_names(&self) -> usize
    {
        *((self.export_directory_addr + 0x20) as *const u32) as usize
    }

    pub unsafe fn addr_of_ordinals(&self) -> usize
    {
        *((self.export_directory_addr + 0x24) as *const u32) as usize
    }

    // List the first 3 bytes of all functions
    // Quite use less right now
    pub unsafe fn list_all_func(&self)
    {
        //let mut index = 0;
        let mut ord = 1;

        for _ in 0..(self.number_of_names())
        {
            //let name_addr = self.base_addr
            //            + *((self.base_addr + self.addr_of_names() + index) as *const u32) as usize;
            //let (name, _) = read_until_null(name_addr as usize);

            let addr = *((self.base_addr + self.addr_of_func() + (ord * 4) ) as *const u32);
            println!("{:x?}", *((self.base_addr + addr as usize) as *const [u8;3]));

            ord += 1;
            //index += 4;
        }
    }

    pub unsafe fn get_func_name(&self, ord: usize) -> String
    {
        let name_addr = self.base_addr
                        + *((self.base_addr + self.addr_of_names() + (ord * 4)) as *const u32) as usize;

        let (name, _) = read_until_null(name_addr as usize);
        let name = String::from_utf8(name).unwrap();

        name
    }

    pub unsafe fn get_func_addr(&self, ord: usize) -> usize
    {
        self.base_addr + *((self.base_addr + self.addr_of_func() + (ord * 4) ) as *const u32) as usize
    }

    pub unsafe fn get_funcs_addr(&self) -> usize
    {
        self.base_addr +  self.addr_of_func() as usize
    }

    pub unsafe fn find_func_addr(&self, find: &str) -> (usize, usize)
    {
        let mut ord = 1;

        // Compute ordinal for given function
        for _ in 0..(self.number_of_names())
        {
            let name = self.get_func_name(ord);
            
            let found = String::from(name);
            let looking_for = String::from(find);

            if found == looking_for
            {
                println!("Found {}", found);
                break
            }

            ord += 1;
        }

        // Use ordinal to get the offset of function
        // And combine it with base address
        let offset = *((self.base_addr + self.addr_of_func() + (ord * 4) ) as *const u32);
        let addr: usize = offset as usize + self.base_addr;

        (addr, ord)
    }
}

/// Iterator implementation
/// Iterate through the ordinal
/// Enable easy iteration over functions / addresses....
impl<'a> IntoIterator for &'a PEImage
{
    type Item = usize;
    type IntoIter = PEImageIntoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PEImageIntoIterator {
            pe: self,
            current_ord: 0,
        }
    }
}

pub struct PEImageIntoIterator<'a>
{
    pe: &'a PEImage,
    current_ord: usize,
}

impl<'a> Iterator for PEImageIntoIterator<'a>
{
    type Item= usize;

    fn next(&mut self) -> Option<Self::Item> 
    {
        unsafe
        {
            if self.current_ord > (self.pe.number_of_func() as usize)
            {
                return None
            }
        }

        self.current_ord += 1;

        Some(self.current_ord - 1)
    }
}

/// Display trait implementation
impl fmt::Display for PEImage 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        unsafe
        { 
            write!(f, "[- {} -]\nBase Addr: {:#x}\nFunc num: {}\nName num: {}\nOptional Header Offset: {:#x}\nExport Directory Addr: {:#x}\nExport Directory Offset: {:#x}\nFunc Offset: {:#x}\nAddr of funcs: {:#x}", self.name, self.base_addr, self.number_of_func(), self.number_of_names(), self.optional_header_offset, self.export_directory_addr, self.export_directory_offset, self.addr_of_func(), self.get_funcs_addr())
        }
    }
}
