use std::path::PathBuf;
use std::fs;
use std::env;
use hashbrown::HashSet;
use std::io::{stdin,stdout};
use std::io::prelude::*;
use colored::*;

//Will return the path of files and subfolder relative to the root path without the root path prefix itself
//Non relative sauce -> https://gist.github.com/srtopster/5a6b5ad7ce5c2b3a9b526d51b87df4f2
fn d_walker_relative(folder: &PathBuf,root_path: &PathBuf) -> HashSet<PathBuf> {
    let mut found = HashSet::new();
    for file in fs::read_dir(&folder).unwrap() {
        let mut file = file.unwrap().path();
        if file.is_dir() {
            found.extend(d_walker_relative(&file,&root_path))
        }else if file.is_file() {
            file = file.strip_prefix(&root_path).unwrap().to_path_buf();
            found.insert(file);
        }
    }
    found
}

fn main() {
    if env::args().len() < 3 {
        println!("Uso: folder_sync <ORIGIN_FOLDER> <DEST_FOLDER>");
        return;
    }
    
    let args: Vec<PathBuf> = env::args().enumerate().filter(|&(i,_)|i>0&&i<3).map(|f|PathBuf::from(f.1)).collect();

    for dir in &args {
        if !dir.is_dir() {
            println!("\"{}\" não existe ou não é uma pasta !",dir.display());
            return;
        }
    }

    let origin_path = PathBuf::from(&args[0]);
    let dest_path = PathBuf::from(&args[1]);

    println!("{}","Descobrindo arquivos...".bright_yellow());
    let origin_files = d_walker_relative(&origin_path,&origin_path);
    let dest_files = d_walker_relative(&dest_path,&dest_path);

    println!("Origem: {} arquivos",origin_files.len());
    println!("Destino: {} arquivos",dest_files.len());

    println!("{}","Detectando diferenças...".bright_yellow());
    let diff: Vec<PathBuf> = origin_files.difference(&dest_files).into_iter().map(|f|f.to_owned()).collect();

    if diff.len() < 1 {
        println!("{}","O destino já possui todos os arquivos da origem !".bright_green());
        return;
    }

    println!("Arquivos não presentes:");
    for file in &diff {
        println!("{}",file.display().to_string().bright_cyan());
    }
    print!("Deseja copiar os arquivos para o destino ? [y/n]: ");
    stdout().flush().unwrap();
    let mut line_buf = String::new();
    stdin().read_line(&mut line_buf).unwrap();
    if line_buf.trim().to_lowercase() == "y" {
        println!("{}","Copiando arquivos...".bright_yellow());
        for (i,file) in diff.iter().enumerate() {
            let origin_file = origin_path.join(&file);
            let dest_file = dest_path.join(&file);
            //Handles missing folders
            //Kinda hacky but it works
            if dest_file.parent().unwrap() != &dest_path {
                fs::create_dir_all(dest_file.parent().unwrap()).expect("Erro ao criar pasta !");
            }
            fs::copy(&origin_file,&dest_file).expect("Falha ao mover !");
            println!("[{}/{}] {} {} {}",format!("{}",i+1).bright_cyan(),diff.len().to_string().bright_cyan(),origin_file.display(),"->".bright_yellow(),dest_file.display());
        }
        println!("{}","Arquivos copiados com sucesso !".bright_green());
    }
}