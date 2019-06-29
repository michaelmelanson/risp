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

type Function = unsafe extern "C" fn() -> u64;

#[derive(Debug)]
pub enum Error {
    MmapError(assembler::ExecutableAnonymousMemoryMapCreationError)
}


pub fn execute<'a, 'b>(term: &Term<'a>) -> Result<u64, Error> {
    let mut memory_map = ExecutableAnonymousMemoryMap::new(4096, false, false).map_err(Error::MmapError)?;

    let hints = assembler::InstructionStreamHints::default();
    let mut stream = memory_map.instruction_stream(&hints);

    let func = compile_term(&mut stream, term);

    let (_bytes, _hints) = stream.finish();
    //print_compiled_code(bytes);
    
    let result = unsafe { func() };
    Ok(result)
}

fn compile_term<'a>(stream: &mut InstructionStream, term: &Term<'a>) -> Function {
    let func = stream.nullary_function_pointer::<u64>();
    stream.push_stack_frame();

    match term {
        Term::Expression(Operator::Add, args) => {
            stream.zero_RAX();

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

                stream.add_Register64Bit_Register64Bit(
                    Register64Bit::RAX,
                    Register64Bit::RDX
                );
            }
        },

        _ => unimplemented!()
    }

    stream.pop_stack_frame_and_return();

    func
}

fn print_compiled_code(bytes: &[u8]) {
    use x86asm::{
        InstructionReader,
        Mode
    };

    let mut reader = InstructionReader::new(bytes, Mode::Long);
    
    loop {
        match reader.read() {
            Ok(instruction) => {
                println!(" - {:?}", instruction);
            },

            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}


