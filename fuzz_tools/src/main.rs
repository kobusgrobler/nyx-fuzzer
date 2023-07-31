extern crate structured_fuzzer;
use clap::{value_t, App, Arg};
use std::fs::File;
use structured_fuzzer::graph_mutator::spec_loader;
use crate::structured_fuzzer::graph_mutator::graph_storage::{VecGraph,GraphStorage};
use crate::structured_fuzzer::graph_mutator::newtypes::{NodeTypeID};
use crate::structured_fuzzer::graph_mutator::spec::{GraphSpec};


fn to_hex_dot(vec: &VecGraph, spec: &GraphSpec) -> String {
        let mut res = "digraph{\n rankdir=\"LR\";\n { edge [style=invis weight=100];".to_string();
        let edges = vec.calc_edges(spec);
        let mut join = "";
        for (_ntype, i) in vec
            .op_iter(&spec)
            .enumerate()
            .filter_map(|(i, op)| op.node().map(|n| (n, i)))
        {
            res += &format!("{}n{}", join, i);
            join = "->";
        }
        res += "}\n";
        for node in vec.node_iter(spec) {
            res += &format!(
                "n{} [label=\"{} {}\", shape=box];\n",
                node.op_i,
                spec.get_node(node.id).unwrap().name,
                hex::encode(node.data)
            );
        }
        for (src, dst) in edges.iter() {
            let node_type = NodeTypeID::new(vec.ops_as_slice()[src.id.as_usize()]);
            let value_type = spec.get_node(node_type).unwrap().outputs[src.port.as_usize()];
            let edge_type = &spec.get_value(value_type).unwrap().name;
            res += &format!(
                "n{} -> n{} [label=\"{}\"];\n",
                src.id.as_usize(),
                dst.id.as_usize(),
                edge_type
            );
        }
        res += "}";
        return res;
}

fn main() {
    let matches = App::new("nyx tools")
    .about("print crashes")
    .arg(
        Arg::with_name("sharedir")
            .short("s")
            .long("sharedir")
            .value_name("SHAREDIR_PATH")
            .takes_value(true)
            .help("path to the sharedir"),
    )
    .arg(
        Arg::with_name("workdir")
            .short("w")
            .long("workdir")
            .value_name("WORKDIR_PATH")
            .takes_value(true)
            .help("overrides the workdir path in the config"),
    )
    .arg(
        Arg::with_name("crash_num")
            .short("n")
            .long("crash_num")
            .value_name("1")
            .takes_value(true)
            .help("crash number"),
    )
    .get_matches();

    let sharedir = matches.value_of("sharedir").expect("need to specify sharedir (-s)").to_string();
    let mut workdir = "/tmp/workdir";
    if let Some(path) = matches.value_of("workdir") {
        workdir = path.clone();
    }

    let crash_num = if let Ok(crash_num) = value_t!(matches, "crash_num", usize) {
        crash_num
    }
    else{
        1
    };

    let spec_path = sharedir+"/spec.msgp";
    if !std::path::Path::new(&spec_path).exists() {
        println!("File not found: {}", spec_path);
        return;
    }
    let file = File::open(spec_path).expect("spec file not found");
    let spec = spec_loader::load_spec_from_read(file);

    let bin_path = format!("{}/corpus/crash/cnt_{}.bin", workdir, crash_num);
    if !std::path::Path::new(&bin_path).exists() {
        println!("File not found: {}", bin_path);
        return;
    }
    
    let vec = VecGraph::new_from_bin_file(&bin_path, &spec);
    // print vec to dot
    println!("{}", to_hex_dot(&vec, &spec));

}
