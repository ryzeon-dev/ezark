use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::ops::Add;
use std::path::Path;
use rsjson::{Json, Node, NodeContent};

pub fn makeArchive(elements: Vec<String>, archiveName: String, verbose: bool) {
    let mutex = std::sync::Mutex::<Vec<String>>::new(Vec::new());
    let checked = std::sync::Arc::<std::sync::Mutex<Vec<String>>>::new(mutex);

    let fileListMutex = std::sync::Mutex::new(Vec::<String>::new());
    let fileListArc = std::sync::Arc::new(fileListMutex);

    let mut jsonMap = rsjson::Json::new();
    let mut archiveIndex: usize = 0;

    for element in elements {
        let elementPath = Path::new(&element);

        if !elementPath.exists() {
            eprintln!("Error: `{}` does not exist", element);
            std::process::exit(1);
        }

        let name = elementPath.file_name().unwrap().to_str().unwrap().to_string();
        if elementPath.is_dir() {

            let (jsonContent, newIndex) = dfs(element.clone(), checked.clone(), archiveIndex, fileListArc.clone(), verbose);
            archiveIndex = newIndex;

            jsonMap.addNode(Node::new(
                name,
                NodeContent::Json(jsonContent)
            ));
        } else {
            if verbose {
                println!("Mapping `{}` file", name);
            }

            fileListArc.lock().unwrap().push(elementPath.to_str().unwrap().to_string());

            let fileBytes = match std::fs::read(elementPath) {
                Err(_) => Vec::new(),
                Ok(bytes) => bytes
            };

            let fileSize = fileBytes.len();
            let mut indexList = Vec::<NodeContent>::new();
            indexList.push(NodeContent::Int(archiveIndex));

            indexList.push(NodeContent::Int(archiveIndex + fileSize));
            archiveIndex += fileSize;

            jsonMap.addNode(Node::new(
                name.clone(),
                NodeContent::List(indexList)
            ));
        }

    }

    let stringSizedJsonMap = jsonMap.toString();

    let mapSizeBytes = stringSizedJsonMap.len().to_string().bytes().collect::<Vec<u8>>();
    let mapBytes = stringSizedJsonMap.bytes().collect::<Vec<u8>>();
    
    if std::path::Path::new(&archiveName).exists() {
        std::fs::remove_file(&archiveName);
    }
    
    std::fs::write(&archiveName, []);
    let mut archiveFile = OpenOptions::new().append(true).open(archiveName).expect("Error: unable to open destination file for writing");

    archiveFile.write(&mapSizeBytes).expect("Error: cannot write to archive file");
    archiveFile.write(&mapBytes).expect("Error: cannot write to archive file");
    
    for file in fileListArc.lock().unwrap().iter() {
        archiveFile.write(&std::fs::read(file).expect(format!("Error: cannot read `{}` file", file).as_str())).expect("Error: cannot write to archive file");
    }
}

fn dfs(base: String, checked: std::sync::Arc<std::sync::Mutex<Vec<String>>>, currentIndex: usize,
    fileListArc: std::sync::Arc<std::sync::Mutex<Vec<String>>>, verbose: bool) -> (rsjson::Json, usize) {
    if checked.lock().unwrap().contains(&base) {
        return (rsjson::Json::new(), currentIndex);
    }

    let mut archiveIndex = currentIndex;

    if verbose {
        println!("Mapping `{}` directory", base);
    }

    let mut node = rsjson::Json::new();

    let dirContentResult = std::fs::read_dir(&base);
    let dirContent;

    match dirContentResult {
        Err(_) => {
            return (rsjson::Json::new(), currentIndex);
        },
        Ok(content) => {
            dirContent = content;
        }
    }

    for e in dirContent {
        let element;
        match e {
            Err(_) => {
                continue;
            },
            Ok(el) => {
                element = el;
            }
        }

        let name = element.file_name().to_str().unwrap().to_string();
        let path = element.path();
        let stringPath = path.to_str().unwrap().to_string();

        if path.is_symlink() {
            continue

        } else if path.is_file() {
            if verbose {
                println!("Mapping `{}` file", stringPath.clone());
            }

            let fileBytes = match std::fs::read(stringPath.clone()) {
                Err(_) => Vec::new(),
                Ok(bytes) => bytes
            };

            fileListArc.lock().unwrap().push(stringPath);

            if verbose {
                println!("Adding `{}/{}` file to archive", base, name.clone());
            }

            let fileSize = fileBytes.len();
            // archiveBytes.lock().unwrap().extend(fileBytes);

            let mut indexList = Vec::<NodeContent>::new();
            indexList.push(NodeContent::Int(archiveIndex));

            indexList.push(NodeContent::Int(archiveIndex + fileSize));
            archiveIndex += fileSize;

            node.addNode(Node::new(
                name.clone(),
                NodeContent::List(indexList)
            ));


        } else if path.is_dir() {
            let (subContent, newIndex) = dfs(stringPath.clone(), checked.clone(), archiveIndex, fileListArc.clone(), verbose);
            archiveIndex = newIndex;

            node.addNode(
                Node::new(
                    name,
                    NodeContent::Json(
                        subContent
                    )
                )
            );
        }
    }

    checked.lock().unwrap().push(base);
    return (node, archiveIndex);
}

fn makeSizedBuffer(size: usize) -> Vec<u8> {
    let mut buffer = Vec::<u8>::new();
    
    for _ in 0..size {
        buffer.push(0);
    }
    
    return buffer;
}

fn extractMap(archiveBytes: &mut BufReader<File>) -> (usize, rsjson::Json){
    let mut buffer = Vec::<u8>::new();
    
    let index = archiveBytes.read_until(123, &mut buffer).unwrap();
    let mut jsonSizeBytes = buffer;
    
    archiveBytes.seek(SeekFrom::Start((index as u64) - 1));
    jsonSizeBytes.remove(jsonSizeBytes.len() - 1);
    
    let size = String::from_utf8(jsonSizeBytes).unwrap().parse::<usize>().unwrap();
    let mut jsonBytes = makeSizedBuffer(size);
    
    archiveBytes.read_exact(&mut jsonBytes);
    return (index, rsjson::Json::fromString(String::from_utf8(jsonBytes.to_vec()).unwrap()).unwrap());
}

pub fn checkArchive(archivePath: String) {
    let mut fileBuffer = BufReader::new(File::open(archivePath).expect("Error: cannot read archive file"));

    let (_, archiveMap) = extractMap(&mut fileBuffer);
    for node in archiveMap.getAllNodes() {
        printArchiveTree(node, 0);
    }
}

pub fn printArchiveTree(node: Node, indent: usize) {
    let indentation = {
        let mut indentString = String::new();

        if indent != 0 {
            let whiteSpace = indent - 4;
            indentString = indentString.add("|   ".repeat(whiteSpace / 4).as_str());
            indentString = indentString.add("|---");
        }

        indentString
    };

    let nodeName = node.getLabel();
    println!("{}{}", indentation, nodeName);

    match node.getContent() {
        NodeContent::Json(subNodes) => {
            for subNode in subNodes.getAllNodes() {
                printArchiveTree(subNode, indent + 4);
            }
        },
        _ => {}
    }
}

pub fn extractArchive(archivePath: String, extractionPath: String, verbose: bool) {
    let mut fileBuffer = BufReader::new(File::open(archivePath).expect("Error: cannot read archive file"));
    let (_, jsonMap) = extractMap(&mut fileBuffer);

    let archiveMutex = std::sync::Mutex::new(fileBuffer);
    let archiveArc = std::sync::Arc::new(archiveMutex);
    
    extractAndCreate(extractionPath, jsonMap, archiveArc.clone(), verbose);
}

fn extractAndCreate(basePath: String, nodes: Json, archive: std::sync::Arc<std::sync::Mutex<BufReader<File>>>, verbose: bool) {
    if !std::path::Path::new(&basePath).exists() {
        match std::fs::create_dir(basePath.clone()) {
            Err(_) => {
                eprintln!("Error: cannot create directory `{}`", basePath);
                std::process::exit(1);
            },
            Ok(_) => {}
        }
    }

    if verbose {
        println!("Extracting `{}` directory", basePath);
    }

    for node in nodes.getAllNodes() {
        let label = node.getLabel();

        match node.getContent() {
            NodeContent::List(list) => {
                let startByte = list.get(0).unwrap().toUsize().unwrap();
                let endByte = list.get(1).unwrap().toUsize().unwrap();

                let mut fileBytes = makeSizedBuffer(endByte - startByte);
                archive.lock().unwrap().read_exact(&mut fileBytes);

                if verbose {
                    println!("Extracting `{basePath}/{label}` file");
                }

                match std::fs::write(format!("{}/{}", basePath.clone(), label.clone()), fileBytes) {
                    Err(_) => {
                        eprintln!("Error: cannot create file `{}/{}`", basePath, label);
                        std::process::exit(1);
                    },
                    Ok(_) => {}
                }
            },

            NodeContent::Json(childNode) => {
                extractAndCreate(
                    format!("{}/{}", basePath.clone(), label), childNode, archive.clone(),
                    verbose
                );
            },

            _ => {}
        }
    }
}