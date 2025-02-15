use ckb_vm::cost_model::estimate_cycles;
use ckb_vm::registers::{A0, A7};
use ckb_vm::{Bytes, CoreMachine, Memory, Register, SupportMachine};

fn debug_syscall<Mac: SupportMachine>(machine: &mut Mac) -> Result<bool, ckb_vm::Error> {
    let code = &machine.registers()[A7];
    if code.to_i32() != 2177 {
        return Ok(false);
    }

    let mut addr = machine.registers()[A0].to_u64();
    let mut buffer = Vec::new();

    loop {
        let byte = machine
            .memory_mut()
            .load8(&Mac::REG::from_u64(addr))?
            .to_u8();
        if byte == 0 {
            break;
        }
        buffer.push(byte);
        addr += 1;
    }

    let s = String::from_utf8(buffer).unwrap();
    println!("{:?}", s);

    Ok(true)
}

#[cfg(has_asm)]
fn main_asm(code: Bytes, args: Vec<Bytes>) -> Result<(), Box<dyn std::error::Error>> {
    let asm_core = ckb_vm::machine::asm::AsmCoreMachine::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_MOP | ckb_vm::ISA_A,
        ckb_vm::machine::VERSION2,
        u64::MAX,
    );
    let core = ckb_vm::DefaultMachineBuilder::new(asm_core)
        .instruction_cycle_func(Box::new(estimate_cycles))
        .syscall(debug_syscall)
        .build();
    let mut machine = ckb_vm::machine::asm::AsmMachine::new(core);
    machine.load_program(&code, &args)?;
    let exit = machine.run();
    let cycles = machine.machine.cycles();
    println!(
        "asm exit={:?} cycles={:?} r[a1]={:?}",
        exit,
        cycles,
        machine.machine.registers()[ckb_vm::registers::A1]
    );
    std::process::exit(exit? as i32);
}

#[cfg(not(has_asm))]
fn main_int(code: Bytes, args: Vec<Bytes>) -> Result<(), Box<dyn std::error::Error>> {
    let core_machine = ckb_vm::DefaultCoreMachine::<u64, ckb_vm::SparseMemory<u64>>::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_MOP | ckb_vm::ISA_A,
        ckb_vm::machine::VERSION2,
        u64::MAX,
    );
    let machine_builder = ckb_vm::DefaultMachineBuilder::new(core_machine)
        .instruction_cycle_func(Box::new(estimate_cycles));
    let mut machine = machine_builder.syscall(debug_syscall).build();
    machine.load_program(&code, &args)?;
    let exit = machine.run();
    let cycles = machine.cycles();
    println!(
        "int exit={:?} cycles={:?} r[a1]={:?}",
        exit,
        cycles,
        machine.registers()[ckb_vm::registers::A1]
    );
    std::process::exit(exit? as i32);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let code = std::fs::read(&args[1])?.into();
    let riscv_args: Vec<Bytes> = if args.len() > 2 {
        (&args[2..]).into_iter().map(|s| s.clone().into()).collect()
    } else {
        Vec::new()
    };
    #[cfg(has_asm)]
    main_asm(code, riscv_args)?;
    #[cfg(not(has_asm))]
    main_int(code, riscv_args)?;
    Ok(())
}
