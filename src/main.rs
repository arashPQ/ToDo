use ncurses::*;
use std::fs::File;
use std::io::{self, Write, BufRead};
use std::env;
use std::process;

const REGULAR_PAIR: i16 = 0;
const HILIGHTED_PAIR: i16 = 1;

type Id = usize;

#[derive(Default)]
struct UI {
    current_list: Option<Id>,
    row: usize,
    col: usize,
}

impl UI {
    fn begin(&mut self, row: usize, col:usize) {
        self.row = row;
        self.col = col;
    }
    
    fn begin_list(&mut self, id: Id) {
        assert!(self.current_list.is_none(), 
        "Nested lists are not allowed!");
        self.current_list = Some(id);
    }

    fn element(&mut self, label: &str, id: Id) -> bool {
        let current_id = self.current_list
        .expect("Not allowed to create list elements outside of lists!!");
        
        let pair = {
            if current_id == id {
                HILIGHTED_PAIR
            } else {
                REGULAR_PAIR
            }
        };

        self.label(label, pair);

        return false;

    }

    fn label(&mut self, text: &str, pair: i16) {
        mv(self.row as i32, self.col as i32);
        attron(COLOR_PAIR(pair));
        let _ = addstr(text);
        attroff(COLOR_PAIR(pair));
        self.row += 1;
    }

    fn end_list(&mut self) {
        self.current_list = None;
    }

    fn end(&mut self) {
    }
}

#[derive(Debug)]
enum Focus {
    TodoTasks,
    DoneTasks,
}

impl Focus {
    fn toggle(&self) -> Self {
        match self {
            Focus::TodoTasks => Focus::DoneTasks,
            Focus::DoneTasks => Focus::TodoTasks,
        }
    }
}

fn parse_item(line: &str) -> Option<(Focus, &str)> {
    let todo_prefix = "TODO: ";
    let done_prefix = "DONE: ";

    if line.starts_with(todo_prefix) {
        return Some((Focus::TodoTasks, &line[todo_prefix.len()..]));
    }
    if line.starts_with(done_prefix) {
        return Some((Focus::DoneTasks, &line[done_prefix.len()..]));
    }
    return None;
}

fn list_up(current_list: &mut usize) {
    if *current_list > 0 {
        *current_list -= 1;
    }
}

fn list_down(list: &Vec<String>, current_list: &mut usize) {
    if *current_list + 1 < list.len() {
        *current_list += 1;
    }
}


fn list_transfer(list_dest: &mut Vec<String>, list_src: &mut Vec<String>, current_src_list: &mut usize) {
    if *current_src_list < list_src.len() {
        list_dest.push(list_src.remove(*current_src_list));
        if *current_src_list >= list_src.len() && list_src.len() > 0 {
            *current_src_list = list_src.len() - 1;
        }
    }
}

fn load_focused(tasks: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) {

    let file = File::open(file_path).unwrap();
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line.unwrap()) {
            Some((Focus::TodoTasks, title)) => tasks.push(title.to_string()),
            Some((Focus::DoneTasks, title)) => dones.push(title.to_string()),
            None => {
                eprintln!("{}:{}: ill.formed item line", file_path, index + 1);
                process::exit(1);
            }
        }
    }
}

fn save_focused(tasks: &Vec<String>, dones: &Vec<String>, file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    for task in tasks.iter() {
        writeln!(file, "TODO: {}", task).unwrap();
    }
    for done in dones.iter() {
        writeln!(file, "DONE: {}", done).unwrap();
    }
}


fn main() {

    let mut args = env::args();
    args.next().unwrap();
    
    let file_path = match args.next() {
        Some(file_path) => file_path,
        None => {
            eprintln!("Usage: TODO <file-path>");
            eprintln!("Error: File path is not provided!!");
            process::exit(1);
        }
    };

    let mut tasks = Vec::<String>::new();
    let mut current_task: usize = 0;
    let mut dones = Vec::<String>::new();
    let mut done_task: usize = 0;

    load_focused(&mut tasks, &mut dones, &file_path);
    

    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    start_color();
    init_pair(REGULAR_PAIR,COLOR_WHITE, COLOR_BLACK);
    init_pair(HILIGHTED_PAIR,COLOR_BLACK, COLOR_WHITE);

    let mut quit = false;

    let mut focus = Focus::TodoTasks;

    let mut ui = UI::default();

    while !quit {
        erase();
        ui.begin(0, 0);
        {
            match focus {
                Focus::TodoTasks => {
                    ui.begin_list(current_task);

                    ui.label("ToDo :", REGULAR_PAIR);
                    ui.label("\n", REGULAR_PAIR);

                    for (index, task) in tasks.iter().enumerate() {
                        ui.element(&format!("- [ ] {}", task),index);
                    }
                    ui.end_list();
                },
                Focus::DoneTasks =>{
                    ui.label("Finished tasks :", REGULAR_PAIR);
                    ui.label("\n", REGULAR_PAIR);
                    
                    ui.begin_list(done_task);
                    for (index, doen_task) in dones.iter().enumerate() {
                        ui.element(&format!("- [x] {}", doen_task),index);
                    }
                    ui.end_list();
                }
            }
            
        }
        ui.end();


        refresh();
        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            'u' => match focus {
                    Focus::TodoTasks => list_up(&mut current_task),
                    Focus::DoneTasks => list_up(&mut done_task),

            },
            'd' => match focus {
                    Focus::TodoTasks => list_down(&tasks, &mut current_task),
                    Focus::DoneTasks => list_down(&dones, &mut done_task),

            }
            '\t' => {
                focus = focus.toggle();
            },
            '\n' => match focus {
                Focus::TodoTasks => list_transfer(&mut dones, &mut tasks, &mut current_task),

                Focus::DoneTasks => list_transfer(&mut tasks, &mut dones, &mut done_task)
            } 
            _ => {}
        }
    }
    save_focused(&tasks, &dones, &file_path);
    endwin();
}
