use std::env;

use todo_cli::TodoList;

fn main() {
    let todo = TodoList::new().expect("Couldn't create the todo intense");

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let command = &args[1];

        match &command[..] {
            "add" => todo.add(&args[2..]),
            "done" => todo.done(&args[2..]),
            "reset" => todo.reset(),
            "remove" => todo.remove(&args[2..]),
            "list" | _ => todo.list(),
        }
    } else {
        todo.list();
    }
}
