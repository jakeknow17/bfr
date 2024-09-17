use crate::parser::Command;

fn is_simple_loop(loop_cmd: &Command) -> bool {
    if let Command::Loop { body, .. } = loop_cmd {
        let mut loop_ptr = 0;
        let mut induction_delta = 0;

        for cmd in body {
            match cmd {
                Command::IncPointer { .. } => loop_ptr += 1,
                Command::DecPointer { .. } => loop_ptr -= 1,
                Command::IncData { .. } => if loop_ptr == 0 { induction_delta += 1 },
                Command::DecData { .. } => if loop_ptr == 0 { induction_delta -= 1 },
                Command::Output { .. } | Command::Input { .. } | Command::Loop { .. } => return false,
            }            
        }
        if loop_ptr == 0 && (induction_delta == -1 || induction_delta == 1) {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
}

pub fn print_profile(commands: &[Command]) {
    struct LoopData {
        idx: usize,
        num_executions: usize,
    }

    fn print_profile_rec(commands: &[Command], curr_idx: &mut usize, simple_loops: &mut Vec<LoopData>, non_simple_loops: &mut Vec<LoopData>) {
        for command in commands {
            match command {
                Command::IncPointer { count } => { 
                    println!("{:>8} : > : {}", curr_idx, count);
                },
                Command::DecPointer { count } => { 
                    println!("{:>8} : < : {}", curr_idx, count);
                },
                Command::IncData { count } => { 
                    println!("{:>8} : + : {}", curr_idx, count);
                },
                Command::DecData { count } => { 
                    println!("{:>8} : - : {}", curr_idx, count);
                },
                Command::Output { count } => { 
                    println!("{:>8} : . : {}", curr_idx, count);
                },
                Command::Input { count } => { 
                    println!("{:>8} : , : {}", curr_idx, count);
                },
                Command::Loop { body, start_count, end_count } => {
                    if is_simple_loop(command) {
                        simple_loops.push(LoopData { idx: *curr_idx, num_executions: *end_count });
                    } else {
                        non_simple_loops.push(LoopData { idx: *curr_idx, num_executions: *end_count });
                    }

                    println!("{:>8} : [ : {}", curr_idx, start_count);
                    *curr_idx += 1;

                    // Recursively print the commands inside the loop
                    print_profile_rec(body, curr_idx, simple_loops, non_simple_loops);

                    println!("{:>8} : ] : {}", curr_idx, end_count);
                },
            }
            *curr_idx += 1;
        }
    }

    // Driver for recursive method
    let mut init_index = 0;
    let mut simple_loops: Vec<LoopData> = vec![];
    let mut non_simple_loops: Vec<LoopData> = vec![];
    print_profile_rec(&commands, &mut init_index, &mut simple_loops, &mut non_simple_loops);

    simple_loops.sort_by(|a, b| b.num_executions.cmp(&a.num_executions));
    non_simple_loops.sort_by(|a, b| b.num_executions.cmp(&a.num_executions));

    for simple_loop in simple_loops {
        println!("Simple loop at index {}, executions: {}", simple_loop.idx, simple_loop.num_executions);
    }
    for non_simple_loop in non_simple_loops {
        println!("Non-simple loop at index {}, executions: {}", non_simple_loop.idx, non_simple_loop.num_executions);
    }
}
