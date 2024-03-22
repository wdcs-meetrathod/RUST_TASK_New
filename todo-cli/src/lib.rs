use colored::*;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
    process,
};

pub struct TodoList {
    pub todo: Vec<String>,
    pub todo_path: String,
    pub todo_backup: String,
}

impl TodoList {
    fn validate_arg(file: &File, arg: &String) {
        let read_buffer = BufReader::new(file);

        for line in read_buffer.lines() {
            match line {
                Ok(l) => {
                    if l.contains(arg) {
                        eprintln!("'{}' is already in todo.", arg);
                        continue;
                    }
                }
                Err(e) => {
                    eprintln!("Error while validating the arg: {}", e);
                    process::exit(1);
                }
            }
        }
    }

    fn not_empty_arg(args: &[String]) {
        if args.is_empty() {
            eprintln!("Please enter at least one todo");
            process::exit(1);
        }
    }
    pub fn new() -> Result<Self, String> {
        let todo_path = match env::var("TODO_PATH") {
            Ok(t) => t,
            Err(_) => {
                let home = env::var("HOME").expect("HOME path should there");
                let legacy_todo = format!("{}/TODO", &home);

                match Path::new(&legacy_todo).exists() {
                    true => {
                        println!("{legacy_todo}");
                        legacy_todo
                    }
                    false => format!("{}/.todo", &home),
                }
            }
        };

        let todo_backup = match env::var("TODO_BACK_DIR") {
            Ok(t) => t,
            Err(_) => String::from("/tmp/todo.bak"),
        };

        let todo_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&todo_path)
            .expect("Couldn't open todo file.");

        let mut buf_reader = BufReader::new(&todo_file);

        let mut contents = String::new();

        buf_reader
            .read_to_string(&mut contents)
            .expect("can read the line from buf");

        let mut todo = vec![];

        for item in contents.lines() {
            if !item.trim().is_empty() {
                todo.push(item.to_string());
            }
        }

        Ok(Self {
            todo,
            todo_backup,
            todo_path,
        })
    }

    pub fn list(&self) {
        let stdio = io::stdout();
        let mut writter = BufWriter::new(stdio);
        let mut data = String::new();

        if self.todo.is_empty() {
            eprintln!("List is empty please add");
            process::exit(1);
        }

        for (index, todo) in self.todo.iter().enumerate() {
            let number = (index + 1).to_string().bold();

            data = format!("{} {}\n\n", number, todo);

            writter
                .write_all(data.as_bytes())
                .expect("can write the list of todo");
        }
    }

    pub fn add(&self, args: &[String]) {
        Self::not_empty_arg(&args);

        let todo_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todo file");

        let file = File::open(&self.todo_path).expect("Couldn't open the file");

        let mut write_buffer = BufWriter::new(&todo_file);

        for arg in args {
            if arg.trim().is_empty() {
                continue;
            }

            Self::validate_arg(&file, arg);

            let line = format!("{}\n", arg);

            write_buffer
                .write_all(line.as_bytes())
                .expect("Couldn't add the todo")
        }
    }

    pub fn done(&self, args: &[String]) {
        Self::not_empty_arg(args);

        let todo_file = OpenOptions::new()
            .write(true)
            .open(&self.todo_path)
            .expect("Couldn't open the file");

        let mut buffer = BufWriter::new(&todo_file);

        for item in self.todo.iter() {
            if args.contains(item) {
                let line = format!("{}\n", &item.strikethrough());

                buffer
                    .write_all(line.as_bytes())
                    .expect("couldn't write the file");
            } else {
                let line = format!("{}\n", &item);
                buffer
                    .write_all(line.as_bytes())
                    .expect("couldn't write the file");
            }
        }
    }
    pub fn reset(&self) {
        match fs::remove_file(&self.todo_path) {
            Ok(_) => println!("Todo has been reset"),
            Err(_) => eprintln!("Couldn't delete the file"),
        };
    }

    pub fn remove(&self, args: &[String]) {
        Self::not_empty_arg(args);

        let todo_file = OpenOptions::new()
            .write(true)
            .open(&self.todo_path)
            .expect("Couldn't open the file");

        let mut buffer = BufWriter::new(&todo_file);

        for item in self.todo.iter() {
            if !args.contains(item) {
                let line = format!("{}\n", &item);
                buffer
                    .write_all(line.as_bytes())
                    .expect("couldn't write the file");
            }
        }
    }
}
