use std::process::{
	Command,
	ExitStatus
};
use std::env;

fn assemble(files: &[&str], target: &str) -> ExitStatus {
	let project_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to retrieve project dir.");
	Command::new("x86_64-w64-mingw32-as")
			.args(files.into_iter().map(|x| project_dir.clone() + "/" + x))
			.arg("-g")
			.arg("-o")
			.arg(target)
			.spawn()
			.expect("Assembling failed")
			.wait()
			.expect("Process running failed")
}

fn main() {
	for (a, b) in env::vars() {
		println!("{} : {}", a, b);
	}
	assert!(assemble(&["src/hw/cpu/smp.asm"], "smp.lib").success());
	assert!(assemble(&["src/kernel/switcher.asm"], "switcher.lib").success());
}
