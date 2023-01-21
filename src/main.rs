mod model;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use clap::Parser;
use crate::model::{FunctionEvent, FunctionHttpEvent, RootServerlessConfig};

#[derive(Parser)]
struct Command {
    from: PathBuf
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c = Command::parse();
    let base_path = &c.from;
    let buf_reader = BufReader::new(File::open(base_path).unwrap());

    let root = serde_yaml::from_reader::<_, RootServerlessConfig>(buf_reader)?;
    let functions = root.functions.instantiate(base_path)?;
    let functions = functions.into_iter().flat_map(|x| x.0).collect::<Vec<_>>();
    // let mut outputs = HashMap::new();
    for function in functions {
        let source_file_location = function.handler.0.source_file_location;
        let identifier = &function.handler.0.identifier;
        let base_file = base_path.parent().unwrap().join(source_file_location);
        println!("base: {base_file}", base_file = &base_file.display());
        let file_name = base_file.file_name().unwrap().to_str().unwrap();
        let d_ts_location = base_file.parent().unwrap().join(format!("{file_name}.d.ts"));
        println!("    processing: {d_ts_location}/{identifier}", d_ts_location = d_ts_location.display());
        let file = File::options().create(true).write(true).append(true).open(d_ts_location).expect("IO Error");
        let mut buf_writer = BufWriter::new(file);
        const BASE_ARGUMENT_TYPE: &str = "{[key: string]: string | number | {} | undefined}";
        const BASE_RETURN_TYPE: &str = "unknown";
        const HTTP_RETURN_TYPE: &str = "Promise<{ statusCode: number, headers: {[key: string]: unknown}, body: string}>";
        let final_type = if let Some(e0) = function.events.get(0) {
            match e0 {
                FunctionEvent::Http(x) => {
                    match &x.http {
                        FunctionHttpEvent::Struct(y) => {
                            let mut constraints = vec![
                                "{ queryParameters: {[query: string]: string | undefined} }".to_string(),
                                "{ pathParameters: {[pathParameterName: string]: string | undefined} }".to_string(),
                            ];

                            let maybe_variable_path = &y.path;
                            let path_variables = maybe_variable_path.split('/')
                                .filter_map(|x| x.strip_prefix('{').and_then(|x| x.strip_suffix('}')));

                            let mut path_constraints = path_variables.map(|v| {
                                format!("{{ queryParameters: {{{v}: string}} }}")
                            }).collect::<Vec<_>>();

                            println!("    detected paths variables: {}", path_constraints.join(" & "));

                            constraints.append(&mut path_constraints);
                            let as_type = constraints.join(" & ");
                            format!("(event: {BASE_ARGUMENT_TYPE} & {as_type}) => {HTTP_RETURN_TYPE}")
                        }
                    }
                }
                FunctionEvent::Unsupported(_) => {
                    println!("    skipped unsupported event");
                    format!("(event: {BASE_ARGUMENT_TYPE}) => {BASE_RETURN_TYPE}")
                }
            }
        } else {
            format!("(event: {BASE_ARGUMENT_TYPE}) => {BASE_RETURN_TYPE}")
        };
        let doc = format!(r#"/** auto generated.
@type {{ {final_type} }}
*/
"#);
        let d = format!(r#"declare const {identifier}: {final_type};
"#);
        use std::io::Write;
        write!(&mut buf_writer, "{}", doc)?;
        write!(&mut buf_writer, "{}", d)?;
        println!("    done.");
    }

    Ok(())
}
