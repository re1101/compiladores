mod cli_options;
mod temp_file;

pub mod compiler;
pub mod preprocessor;

use cli_options::*;
use temp_file::*;
use compiler::{
    codegen::register_allocation::*, codegen::*, common::error::*, parser::*, scanner::*, typechecker::*,
};
use preprocessor::{scanner::Scanner as PPScanner, *};

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Preprocesses given input file by converting String into preprocessor-tokens.<br>
pub fn preprocess(
    filename: &Path,
    user_include_dirs: &Vec<PathBuf>,
    defines: &Vec<(String, String)>,
    //standard_headers: &HashMap<PathBuf, &'static str>,
    source: String,
) -> Result<Vec<PPToken>, WreccError> {
    let tokens = PPScanner::new(source).scan_token();
    let include_depth = 0;

    // INFO: convert all cli-passed defines to #defines as if they were in regular source file
    // to properly error check them
    let mut dummy_defines = String::new();
    for (macro_name, value) in defines {
        dummy_defines.push_str(&format!("#define {} {}\n", macro_name, value));
    }
    let (_, defines) = Preprocessor::new(
        &PathBuf::from("command-line-argument"),
        PPScanner::new(dummy_defines).scan_token(),
        HashMap::new(),
        user_include_dirs,
        //standard_headers,
        include_depth,
    )
    .start()
    .map_err(|errors| WreccError::Cli(errors.iter().map(|e| e.kind.message()).collect()))?;

    Ok(Preprocessor::new(
        filename,
        tokens,
        defines,
        user_include_dirs,
        //standard_headers,
        include_depth,
    )
    .start()
    .map(|(tokens, _)| tokens)?)
}

/// Compiles preprocessor-tokens to a x86-64 string, using functionality defined in [compiler]
pub fn compile(source: Vec<PPToken>, dump_ast: bool) -> Result<String, WreccError> {
    let tokens = Scanner::new(source).scan_token()?;

    let parse_tree = Parser::new(tokens).parse()?;

    if dump_ast {
        parse_tree.iter().for_each(|decl| eprintln!("{}", decl));
    }

    let (mir, const_labels) = TypeChecker::new().check(parse_tree)?;

    let (lir, live_intervals) = Compiler::new(const_labels).translate(mir);

    let asm = RegisterAllocation::new(live_intervals).generate(lir);

    let output = asm
        .into_iter()
        .map(|instr| instr.as_string())
        .collect::<Vec<String>>()
        .join("\n");

    Ok(output)
}

fn generate_asm_file(options: &CliOptions, file: &Path, output: String) -> Result<OutFile, WreccError> {
    let output_path = output_path(file, &options.output_path, options.compile_only, "s");

    let mut output_file = std::fs::File::create(output_path.get()).map_err(|_| {
        WreccError::Sys(format!("could not create file '{}'", output_path.get().display()))
    })?;

    if writeln!(output_file, "{}", output).is_err() {
        Err(WreccError::Sys(format!(
            "could not write to file '{}'",
            output_path.get().display()
        )))
    } else {
        Ok(output_path)
    }
}

fn output_path(
    file: &Path,
    output_path: &Option<PathBuf>,
    is_last_phase: bool,
    extension: &'static str,
) -> OutFile {
    match (output_path, is_last_phase) {
        (Some(file), true) => OutFile::Regular(file.clone()),
        (None, true) => OutFile::Regular(file.with_extension(extension)),
        (_, false) => OutFile::Temp(TempFile::new(extension)),
    }
}

fn read_input_file(file: &Path) -> Result<String, WreccError> {
    fs::read_to_string(file)
        .map_err(|_| WreccError::Sys(format!("could not find file: '{}'", file.display())))
}

fn print_pp(pp_source: Vec<PPToken>, options: &CliOptions) -> Result<(), WreccError> {
    let pp_string: String = pp_source.iter().map(|s| s.kind.to_string()).collect();

    if let Some(pp_file) = &options.output_path {
        let mut output_file = std::fs::File::create(pp_file.clone())
            .map_err(|_| WreccError::Sys(format!("could not create file '{}'", pp_file.display())))?;

        if writeln!(output_file, "{}", pp_string).is_err() {
            Err(WreccError::Sys(format!(
                "could not write to file '{}'",
                pp_file.display()
            )))
        } else {
            Ok(())
        }
    } else {
        eprintln!("{}", pp_string);
        Ok(())
    }
}

fn process_file(options: &CliOptions) -> Result<Option<OutFile>, WreccError> {
    let file = Path::new("../prueba.txt");

    let source = read_input_file(&file)?;

    let pp_source = preprocess(
        &file,
        &Vec::new(),
        &Vec::new(),
        //standard_headers,
        source,
    )?;

    //print_pp(pp_source, &options)?;

    let asm_source = compile(pp_source, false)?;

    let _asm_file = generate_asm_file(&options, file, asm_source)?;

    Ok(None)
}

fn run(options: CliOptions) -> Result<(), Vec<WreccError>> {
    let mut errors = Vec::new();

    match process_file(&options) {
        Ok(_) => (),
        Err(e) => errors.push(e),
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(())
}

fn rm()  -> Result<(), ()>{
    let mut options = CliOptions::new();
    options.output_path = Some(PathBuf::from("../resultado.txt"));

    let no_color = options.no_color;

    run(options).map_err(|errs| errs.into_iter().map(|e| e.print(no_color)).collect())
}

fn main() {
    match rm() {
        Ok(_) => (),
        Err(_) => std::process::exit(1),
    }
}