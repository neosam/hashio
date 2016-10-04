#[macro_use]
extern crate hashio;
#[macro_use]
extern crate log;

use hashio::io::*;
use hashio::hashio::*;
use std::io::{Read, Write};
use std::{io};
use hashio::hash::*;
use std::collections::BTreeMap;
use std::result;
use std::rc::Rc;
use hashio::hashiofile::HashIOFile;
use std::fs::remove_dir_all;

hashio_type! {
	Task1 {
		factor: f32, read_f32, write_f32
	} {
		title: String
	}
}
hashio_type! {
	Task {
		factor: f32, read_f32, write_f32
	} {
		title: String,
		category: String
	}
}
impl From<Rc<Task1>> for Task {
	fn from(old: Rc<Task1>) -> Task {
		Task {
			factor: old.factor,
			title: old.title.clone(),
			category: Rc::new("".to_string())
		}
	}
}

hashio_type! {
	TaskStrage1 {
	} {
		tasks: Vec<Rc<Task1>>
	}
}
hashio_type! {
	TaskStrage {
	} {
		tasks: Vec<Rc<Task>>
	}
	fallback => TaskStrage1
}
impl From<Rc<TaskStrage1>> for TaskStrage {
	fn from(old: Rc<TaskStrage1>) -> TaskStrage {
		let mut tasks: Vec<Rc<Task>> = Vec::new();
		for task in old.tasks.iter() {
			tasks.push(Rc::new(Task::from(task.clone())));
		}
		TaskStrage {
			tasks: Rc::new(tasks)
		}
	}
}

#[test]
fn test() {
	remove_dir_all("unittest/overalltest").ok();
	let hash_io = HashIOFile::new("unittest/overalltest".to_string());
	let task1 = Task1 {title: Rc::new("Test1".to_string()), factor: 0.5};
	let task2 = Task1 {title: Rc::new("Test2".to_string()), factor: 0.2};
	let storage = TaskStrage1 { tasks: Rc::new(vec![Rc::new(task1), Rc::new(task2)]) };
	let hash = storage.as_hash();
	hash_io.put(Rc::new(storage)).unwrap();

	let storage: Rc<TaskStrage> = hash_io.get(&hash).unwrap();
	assert_eq!(Rc::new("Test1".to_string()), storage.tasks[0].title.clone());
	assert_eq!(Rc::new("Test2".to_string()), storage.tasks[1].title.clone());
	assert_eq!(Rc::new("".to_string()), storage.tasks[0].category.clone());
	assert_eq!(Rc::new("".to_string()), storage.tasks[1].category.clone());
}

