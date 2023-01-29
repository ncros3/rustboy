mod bus;
mod cpu;
mod gpu;
mod nvic;

use cpu::Cpu;

fn main() {
    // create the emulated system
    let mut cpu = Cpu::new();

    // run the emulated system
    cpu.run();

    println!("hello rustboy !");
}
