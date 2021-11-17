mod bus;
mod cpu;
mod gpu;

use cpu::Cpu;

fn main() {
    let cpu = Cpu::new();

    println!("hello rustboy !");
}
