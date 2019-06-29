use crate::parser::{
    Operator,
    Literal,
    Term
};

use assembler::{
    mnemonic_parameter_types::{
        immediates::*,
        registers::*
    },
    ExecutableAnonymousMemoryMap,
    InstructionStream
};

type Function = unsafe extern "C" fn() -> i64;

#[derive(Debug)]
pub enum Error {
    MmapError(assembler::ExecutableAnonymousMemoryMapCreationError)
}


pub fn execute<'a>(term: &Term<'a>) -> Result<Literal<'a>, Error> {
    let mut memory_map = ExecutableAnonymousMemoryMap::new(4096, false, false).map_err(Error::MmapError)?;

    let hints = assembler::InstructionStreamHints::default();
    let mut stream = memory_map.instruction_stream(&hints);

    let func = compile_term(&mut stream, term);

    stream.finish();
    
    let result = unsafe { func() };

    Ok(Literal::Int(result))
}

fn compile_term<'a>(stream: &mut InstructionStream, term: &Term<'a>) -> Function {
    let func = stream.nullary_function_pointer::<i64>();
    stream.push_stack_frame();

    match term {
        Term::Expression(operator, args) => {
            match operator {
                Operator::Add => stream.zero_RAX(),
                Operator::Multiply => stream.mov_Register64Bit_Immediate64Bit(
                    Register64Bit::RAX,
                    Immediate64Bit(1)
                ),
                _ => {}
            }

            for arg in args {
                match arg {
                    Term::Literal(Literal::Int(int)) => {
                        stream.mov_Register64Bit_Immediate64Bit(
                            Register64Bit::RDX,
                            Immediate64Bit(*int)
                        );
                    },

                    _ => unimplemented!()
                }

                match operator {
                    Operator::Add => {
                        stream.add_Register64Bit_Register64Bit(
                            Register64Bit::RAX,
                            Register64Bit::RDX
                        );
                    },

                    Operator::Multiply => {
                        stream.imul_Register64Bit_Register64Bit(
                            Register64Bit::RAX,
                            Register64Bit::RDX
                        );
                    },

                    Operator::CallFunction(_) => unimplemented!()
                }
            }
        },

        _ => unimplemented!()
    }

    stream.pop_stack_frame_and_return();

    func
}

// fn print_compiled_code(bytes: &[u8]) {
//     use x86asm::{
//         InstructionReader,
//         Mode
//     };

//     let mut reader = InstructionReader::new(bytes, Mode::Long);
    
//     loop {
//         match reader.read() {
//             Ok(instruction) => {
//                 println!(" - {:?}", instruction);
//             },

//             Err(err) => {
//                 println!("Error: {:?}", err);
//                 break;
//             }
//         }
//     }
// }


