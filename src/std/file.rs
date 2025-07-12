use crate::virt::fs;

pub fn read_file(disk_id: u64, path: FilePath) -> Box<[u8]> {

}

pub fn mount(disk_id: u64) -> bool {
	fs::Mountpoint::from_disk(disk_id).mount();
}
