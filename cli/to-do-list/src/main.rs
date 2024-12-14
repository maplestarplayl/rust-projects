use std::fs;
use std::io::{BufRead,BufReader, Write};
fn main() {
    println!("Hello, world!");
    let mut todo_list = load_todos().unwrap_or_else(|_| Vec::new());
    loop {
        println!("1. Add Todo");
        println!("2. List Todos");
        println!("3. Remove Todo");
        println!("4. Save and Exit");
        let choice = read_line();

        match choice.as_str() {
            "1" => add_todo(&mut todo_list),
            "2" => list_todos(&todo_list),
            "3" => remove_todo(&mut todo_list),
            "4" => {
                save_todos(&todo_list).expect("Failed to save todos");
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }
}



fn read_line() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("failde to read line");
    input.trim().to_string()
}
fn load_todos() -> std::io::Result<Vec<String>> {
    let file = fs::File::open("todos.txt")?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

fn save_todos(todos: &[String]) -> std::io::Result<()> {
    let mut file = fs::File::create("todos.txt")?;
    for todo in todos {
        file.write_all(todo.as_bytes())?;
    }
    Ok(())
}

fn add_todo(todos: &mut Vec<String>) {
    let todo = read_line();
    if !todo.is_empty() {
        todos.push(todo);
        println!("Todo added successfully");
    }
}

fn list_todos(todos: &[String]) {
    if todos.is_empty() {
        println!("No todos found");
    } else {
        for (index, todo) in todos.iter().enumerate() {
            println!("{}. {}", index + 1, todo);
        }
    }
}

fn remove_todo(todos: &mut Vec<String>) {
    list_todos(todos);
    if !todos.is_empty() {
        println!("Enter the number of the todo to remove:");
        let index = read_line().parse::<usize>().expect("Invalid input");
        if index > 0 && index <= todos.len() {
            todos.remove(index - 1);
            println!("Todo removed successfully");
        } else {
            println!("Invalid todo number");
        }
    }
}