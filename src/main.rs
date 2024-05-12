use std::path::PathBuf;
use std::fs;
use std::env;
use hashbrown::HashSet;
use colored::*;
use inquire::{MultiSelect,validator::Validation,list_option::ListOption};

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
    println!("{}","=".repeat(75));
    
    let origin_files = d_walker_relative(&origin_path,&origin_path);
    let dest_files = d_walker_relative(&dest_path,&dest_path);

    println!("Origem: {} arquivos",origin_files.len());
    println!("Destino: {} arquivos",dest_files.len());

    println!("{}","=".repeat(75));
    println!("{}","Detectando diferenças...".bright_yellow());
    println!("{}","=".repeat(75));

    let to_copy: Vec<PathBuf> = origin_files.difference(&dest_files).into_iter().map(|f|f.to_owned()).collect();
    let to_delete: Vec<PathBuf> = dest_files.difference(&origin_files).into_iter().map(|f|f.to_owned()).collect();

    let mut options = Vec::new();

    if !to_copy.is_empty() {
        options.push("Copiar arquivos novos");
        println!("[{}] Arquivos para copiar:",to_copy.len().to_string().bright_yellow());
        for file in &to_copy {
            println!(" | {}",file.display().to_string().bright_cyan());
        }
    }

    if !to_delete.is_empty() {
        options.push("Deletar diferenças no destino");
        println!("[{}] Arquivos para deletar:",to_delete.len().to_string().bright_yellow());
        for file in &to_delete {
            println!(" | {}",file.display().to_string().bright_red());
        }
    }

    options.push("Sair");

    if options.len() == 1 {
        println!("{}","Nada para ser feito !".bright_green());
        return;
    }

    println!("{}","=".repeat(75));
    let validator = |a: &[ListOption<&&str>]| {
        if a.is_empty() {
            return Ok(Validation::Invalid("Selecione pelo menos uma opção !".into()))
        } else {
            return Ok(Validation::Valid);
        }
    };

    let choice = MultiSelect::new("O que deseja fazer ?", options).with_help_message("↑↓ para mover, espaço para selecionar, enter para prosseguir").with_validator(validator).prompt().unwrap();

    println!("{}","=".repeat(75));

    if choice.contains(&"Sair") {
        print!("Saindo...");
        return;
    }

    if choice.contains(&"Copiar arquivos novos") {
        println!("{}","Copiando arquivos...".bright_yellow());
        for (i,file) in to_copy.iter().enumerate() {
            let origin_file = origin_path.join(&file);
            let dest_file = dest_path.join(&file);
            //Handles missing folders
            //Kinda hacky but it works
            if dest_file.parent().unwrap() != &dest_path {
                fs::create_dir_all(dest_file.parent().unwrap()).expect("Erro ao criar pasta !");
            }
            fs::copy(&origin_file,&dest_file).expect("Falha ao mover !");
            println!("[{}/{}] {} {} {}",format!("{}",i+1).bright_cyan(),to_copy.len().to_string().bright_cyan(),origin_file.display(),"->".bright_yellow(),dest_file.display());
        }
        println!("{}","Arquivos copiados com sucesso !".bright_green())
    }

    if choice.contains(&"Deletar diferenças no destino") {
        println!("{}","Deletando aquivos...".bright_yellow());
        for (i,file) in to_delete.iter().enumerate() {
            let dest_file = dest_path.join(&file);
            fs::remove_file(&dest_file).expect("Falha ao deletar !");
            println!("[{}/{}]Deletado: {}",format!("{}",i+1).bright_cyan(),to_delete.len().to_string().bright_cyan(),dest_file.display().to_string().bright_red());
        }
        println!("{}","Arquivos deletados com sucesso !".bright_green())
    }
}