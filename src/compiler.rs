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

pub struct Function {
    memory_map: Box<ExecutableAnonymousMemoryMap>,
    func: unsafe extern "C" fn() -> i64
}

impl Function {
    pub fn call(&self) -> Literal {
        let result = unsafe { (self.func)() };

        Literal::Int(result)
    }
}

#[derive(Debug)]
pub enum Error {
    MmapError(assembler::ExecutableAnonymousMemoryMapCreationError)
}


pub fn compile(term: &Term) -> Result<Function, Error> {
    let mut memory_map = Box::new(ExecutableAnonymousMemoryMap::new(4096, false, false).map_err(Error::MmapError)?);

    let hints = assembler::InstructionStreamHints::default();
    let mut stream = memory_map.instruction_stream(&hints);

    let func = stream.nullary_function_pointer::<i64>();
    stream.push_stack_frame();
    compile_term(&mut stream, Register64Bit::RAX, term);
    stream.pop_stack_frame_and_return();

    stream.finish();
    
    Ok(Function {
        memory_map,
        func
    })
}

fn compile_term(stream: &mut InstructionStream, destination: Register64Bit, term: &Term) {
    match term {
        Term::Expression(operator, args) => compile_expression(stream, destination, operator, args),

        _ => unimplemented!()
    }
}

fn compile_expression(
    stream: &mut InstructionStream,
    destination: Register64Bit,
    operator: &Operator,
    args: &Vec<Term>
) {
    let intermediate_register = Register64Bit::R10;
    let scratch_register = Register64Bit::R11;

    stream.push_Register64Bit_r64(intermediate_register);

    // Clear the destination register
    match operator {
        Operator::Add => stream.mov_Register64Bit_Immediate64Bit(
            destination,
            Immediate64Bit(0)
        ),
        Operator::Multiply => stream.mov_Register64Bit_Immediate64Bit(
            destination,
            Immediate64Bit(1)
        ),
        Operator::CallFunction(_func) => unimplemented!()
    }

    for arg in args {
        match arg {
            Term::Literal(Literal::Int(int)) => {
                stream.mov_Register64Bit_Immediate64Bit(
                    intermediate_register,
                    Immediate64Bit(*int)
                );
            },
            
            Term::Literal(Literal::Str(_str)) => unimplemented!(),

            Term::Expression(operator, args) => {
                stream.push_Register64Bit_r64(scratch_register);
                compile_expression(stream, scratch_register, operator, args);
                stream.mov_Register64Bit_Register64Bit_r64_rm64(
                    intermediate_register,
                    scratch_register
                );
                stream.pop_Register64Bit_r64(scratch_register);
            },

            Term::Identifier(_identifier) => unimplemented!() 
        }

        match operator {
            Operator::Add => {
                stream.add_Register64Bit_Register64Bit(
                    destination,
                    intermediate_register
                );
            },

            Operator::Multiply => {
                stream.imul_Register64Bit_Register64Bit(
                    destination,
                    intermediate_register
                );
            },

            Operator::CallFunction(_) => unimplemented!()
        }
    }

    stream.pop_Register64Bit_r64(intermediate_register);
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


