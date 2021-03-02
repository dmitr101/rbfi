use std::io;
use std::io::Read;
use std::fs;
use std::env;
use std::result::Result;

const MAX_MEM_CELL_COUNT : usize = 1024;

struct InterpreterContext  {
    memory : [i8; MAX_MEM_CELL_COUNT],
    mem_cell_index : usize,
    cycle_stack : Vec<usize>,
    script_cursor : usize,
}

#[derive(Debug)]
enum ExecutionError{
    MemCellAccessOutOfBounds{cell_idx: usize, script_pos: usize},
    MemCellValueOverflow{cell_idx: usize, script_pos: usize},
    MemCellValueUnderflow{cell_idx: usize, script_pos: usize},
    UnexpectedCycleEnd{script_pos: usize},
    IOError{script_pos: usize},
}

impl InterpreterContext {
    fn new() -> InterpreterContext {
        InterpreterContext {
            memory : [0; MAX_MEM_CELL_COUNT],
            mem_cell_index : 0,
            cycle_stack : Vec::new(),
            script_cursor : 0,
        }
    }

    fn check_out_of_bounds_access(&self) -> Result<(), ExecutionError> {
        if self.mem_cell_index >= MAX_MEM_CELL_COUNT {
            return Err(ExecutionError::MemCellAccessOutOfBounds{
                cell_idx: self.mem_cell_index, 
                script_pos: self.script_cursor,
            });
        }
        Ok(())
    }

    fn inc_checked(&mut self) -> Result<(), ExecutionError> {
        self.check_out_of_bounds_access()?;
        let cur_cell = &mut self.memory[self.mem_cell_index];
        match cur_cell.checked_add(1) {
            Some(v) => *cur_cell = v,
            None => return Err(ExecutionError::MemCellValueOverflow{
                cell_idx: self.mem_cell_index, 
                script_pos: self.script_cursor,
            })
        }
        Ok(())
    }
    
    fn dec_checked(&mut self) -> Result<(), ExecutionError> {
        self.check_out_of_bounds_access()?;
        let cur_cell = &mut self.memory[self.mem_cell_index];
        match cur_cell.checked_sub(1) {
            Some(v) => *cur_cell = v,
            None => return Err(ExecutionError::MemCellValueUnderflow{
                cell_idx: self.mem_cell_index, 
                script_pos: self.script_cursor,
            })
        }
        Ok(())
    }
    
    fn put_checked(&self) -> Result<(), ExecutionError> {
        self.check_out_of_bounds_access()?;
        print!("{}", self.memory[self.mem_cell_index] as u8 as char);
        Ok(())
    }

    fn get_checked(&mut self) -> Result<(), ExecutionError> {
        self.check_out_of_bounds_access()?;   
        match get_one_byte_from_stdin() {
            Some(c) => self.memory[self.mem_cell_index] = c,
            None => return Err(ExecutionError::IOError{
                script_pos: self.script_cursor
            })
        }
        Ok(())
    }
    
    fn execute(&mut self, script : &str) -> Result<(), ExecutionError>{
        loop {
            match script.as_bytes()[self.script_cursor] {
                b'>' => self.mem_cell_index += 1,
                b'<' => self.mem_cell_index -= 1,
                b'+' => self.inc_checked()?,
                b'-' => self.dec_checked()?,
                b'.' => self.put_checked()?,
                b',' => self.get_checked()?,
                b'[' => {
                    self.check_out_of_bounds_access()?;
                    if self.memory[self.mem_cell_index] != 0 {
                        self.cycle_stack.push(self.script_cursor);
                    } else {
                        match script.split_at(self.script_cursor).1.find(']') {
                            Some(offset) => self.script_cursor += offset,
                            None => self.script_cursor = script.len(),
                        }
                    }
                },
                b']' => {
                    if !self.cycle_stack.is_empty() {
                        self.script_cursor = self.cycle_stack.pop().unwrap() - 1;
                    } else {
                        return Err(ExecutionError::UnexpectedCycleEnd{
                            script_pos: self.script_cursor
                        });
                    }
                },
                _ => {}
            }
    
            self.script_cursor += 1;
            if self.script_cursor >= script.len() {
                break;
            }
        }
        Ok(())
    }

    fn execute_from_start(&mut self, script : &str) -> Result<(), ExecutionError> {
        self.script_cursor = 0;
        self.execute(script)
    }
}

fn get_one_byte_from_stdin() -> Option<i8> {
    io::stdin()
        .bytes()
        .next()
        .and_then(|result| result.ok())
        .map(|byte| byte as i8)
}

fn main() -> io::Result<()> {
    if let Some(arg) = env::args().nth(1) {
        let script_str = {
            let script_file = fs::File::open(arg.as_str())?;
            let mut reader = io::BufReader::new(script_file);
            let mut content = String::new();
            reader.read_to_string(&mut content).unwrap();
            content
        };

        let mut context = InterpreterContext::new();
        match context.execute_from_start(script_str.as_str()) {
            Ok(()) => println!("\nExecutted successfully. Exitting..."),
            Err(e) => println!("\nError {:?} occured!", e),
        }
    } else {
        println!("Error: Expected to get a script filename as an argument!")
    }
    Ok(())
}