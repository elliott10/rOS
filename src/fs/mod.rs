mod device;

use lazy_static::*;
use rcore_fs::vfs::*;
use rcore_fs_sfs::SimpleFileSystem;
use alloc::{ sync::Arc, vec::Vec };

lazy_static! {
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        // 创建内存模拟的"磁盘"设备
        let device = {
            extern "C" {
                fn _user_img_start();
                fn _user_img_end();
            }

            let start = _user_img_start as usize;
            let end = _user_img_end as usize;
            Arc::new(unsafe { device::MemBuf::new(start, end) })
        };

        // SimpleFileSystem 打开该设备进行初始化
        let sfs = SimpleFileSystem::open(device).expect("failed to open SimpleFS");
        sfs.root_inode()
    };
}

pub trait InodeExt {
    fn read_as_vec(&self) -> Result<Vec<u8>>;
}

impl InodeExt for dyn INode {
    fn read_as_vec(&self) -> Result<Vec<u8>> {
        let size = self.metadata()?.size;
        let mut buf = Vec::with_capacity(size);
        unsafe { buf.set_len(size); }
        self.read_at(0, buf.as_mut_slice())?;
        Ok(buf)
    }
}

pub fn init() {
    println!("available programs in rust/ are:");
    let mut id = 0;
    let mut rust_dir = ROOT_INODE.lookup("rust").unwrap();
    while let Ok(name) = rust_dir.get_entry(id) {
        id += 1;
        println!(" {}", name);
    }
    println!("+++ setup fs! +++");
}

