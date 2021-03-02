use std::io;
use std::io::Read;
use std::fs;
use std::env;
use std::result::Result;

const MAX_MEM_CELL_COUNT : usize = 1024;

struct InterpreterContext  {
    memory : [i8; MAX_MEM_CELL_COUNT],
    index : usize,
}

impl InterpreterContext {
     fn new() -> InterpreterContext {
        InterpreterContext {
            memory : [0; MAX_MEM_CELL_COUNT],
            index : 0
        }
    }
}

enum ExecutionError{
    MemCellAccessOutOfBounds(usize, usize),
    MemCellValueOverflow(usize, usize),
    MemCellValueUnderflow(usize, usize),
    UnexpectedCycleEnd(usize),
    IOError,
}

//TODO: Make it easier to use
fn exec_checked<F>(op : F, context : &mut InterpreterContext, pos : usize, err : ExecutionError) -> Result<(), ExecutionError> 
    where F: FnOnce(&mut InterpreterContext) -> bool {
    if context.index >= MAX_MEM_CELL_COUNT { 
        return Err(ExecutionError::MemCellAccessOutOfBounds(context.index, pos));
    }

    if op(context) {
        return Ok(())
    }
    Err(err)
}

fn inc_checked(context : &mut InterpreterContext, pos : usize) -> Result<(), ExecutionError> {
    if context.index >= MAX_MEM_CELL_COUNT {
        return Err(ExecutionError::MemCellAccessOutOfBounds(context.index, pos));
    }
    if context.memory[context.index] == i8::MAX {
        return Err(ExecutionError::MemCellValueOverflow(context.index, pos));
    }

    context.memory[context.index] += 1;
    Ok(())
}

fn dec_checked(context : &mut InterpreterContext, pos : usize) -> Result<(), ExecutionError> {
    if context.index >= MAX_MEM_CELL_COUNT {
        return Err(ExecutionError::MemCellAccessOutOfBounds(context.index, pos));
    }
    if context.memory[context.index] == i8::MIN {
        return Err(ExecutionError::MemCellValueUnderflow(context.index, pos));
    }

    context.memory[context.index] -= 1;
    Ok(())
}

fn put_checked(context : &mut InterpreterContext, pos : usize) -> Result<(), ExecutionError> {
    if context.index >= MAX_MEM_CELL_COUNT {
        return Err(ExecutionError::MemCellAccessOutOfBounds(context.index, pos));
    }

    print!("{}", context.memory[context.index] as u8 as char);
    Ok(())
}

fn get_checked(context : &mut InterpreterContext, pos : usize) -> Result<(), ExecutionError> {
    if context.index >= MAX_MEM_CELL_COUNT {
        return Err(ExecutionError::MemCellAccessOutOfBounds(context.index, pos));
    }

    match io::stdin()
        .bytes()
        .next()
        .and_then(|result| result.ok())
        .map(|byte| byte as i8) {
        Some(c) => context.memory[context.index] = c,
        None => return Err(ExecutionError::IOError)
    }
    Ok(())
}

fn execute(script : &str, context : &mut InterpreterContext) -> Result<(), ExecutionError> {
    let mut script_cursor : usize = 0;
    let mut cycle_stack : Vec<usize> = Vec::new();
    loop {
        match script.as_bytes()[script_cursor] {
            b'>' => context.index += 1,
            b'<' => context.index -= 1,
            b'+' => inc_checked(context, script_cursor)?,
            b'-' => dec_checked(context, script_cursor)?,
            b'.' => put_checked(context, script_cursor)?,
            b',' => get_checked(context, script_cursor)?,
            b'[' => {
                if context.memory[context.index] != 0 {
                    cycle_stack.push(script_cursor);
                } else {
                    match script.split_at(script_cursor).1.find(']') {
                        Some(offset) => script_cursor += offset,
                        None => script_cursor = script.len(),
                    }
                }
            },
            b']' => {
                if !cycle_stack.is_empty() {
                    script_cursor = cycle_stack.pop().unwrap() - 1;
                } else {
                    return Err(ExecutionError::UnexpectedCycleEnd(script_cursor));
                }
            },
            _ => {}
        }

        script_cursor += 1;
        if script_cursor >= script.len() {
            break;
        }
    }
    Ok(())
}

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Interpreter expects one argument only - a file with a script!");
        return;
    }

    let arg_path = args[1].as_str();
    let script_file = match fs::File::open(arg_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Couldn't open file {}, error: {}", arg_path, e);
            return;
        }
    };

    let script_str = {
        let mut reader = io::BufReader::new(script_file);
        let mut content = String::new();
        reader.read_to_string(&mut content).unwrap();
        content
    };

    let mut context = InterpreterContext::new();
    match execute(script_str.as_str(), &mut context) {
        Ok(()) => println!("Executted successfully. Exitting..."),
        Err(e) => match e {
            ExecutionError::MemCellAccessOutOfBounds(s1,s2) => println!("Error. Overflowing {}", s1),
            ExecutionError::MemCellValueOverflow(s1, s2) => println!("Error. Overflowing {}", s1),
            ExecutionError::MemCellValueUnderflow(s1, s2) => println!("Error. Overflowing {}", s1),
            ExecutionError::UnexpectedCycleEnd(s) => println!("Error. Unexpected cycle end at position {}", s),
            ExecutionError::IOError => println!("Error. Couldn't read from the terminal."),
        }
    }
}
