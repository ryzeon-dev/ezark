use std::ops::Add;
use std::path::Path;
use rsjson::{Json, Node, NodeContent};

pub fn makeArchive(elements: Vec<String>, archiveName: String, verbose: bool) {
    let mutex = std::sync::Mutex::<Vec<String>>::new(Vec::new());
    let checked = std::sync::Arc::<std::sync::Mutex<Vec<String>>>::new(mutex);

    let archiveBytesMutex = std::sync::Mutex::new(Vec::<u8>::new());
    let archiveBytesArc = std::sync::Arc::new(archiveBytesMutex);

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

            let (jsonContent, newIndex) = dfs(element.clone(), checked.clone(), archiveIndex, archiveBytesArc.clone(), verbose);
            archiveIndex = newIndex;

            jsonMap.addNode(Node::new(
                name,
                NodeContent::Json(jsonContent)
            ));
        } else {
            if verbose {
                println!("Mapping `{}` file", name);
            }

            let fileBytes = match std::fs::read(elementPath) {
                Err(_) => Vec::new(),
                Ok(bytes) => bytes
            };

            if verbose {
                println!("Adding `{}` file to archive", name.clone());
            }

            let fileSize = fileBytes.len();
            archiveBytesArc.lock().unwrap().extend(fileBytes);

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

    let mut fileBytes = Vec::<u8>::new();
    fileBytes.extend(mapSizeBytes);

    fileBytes.extend(mapBytes);
    fileBytes.extend(archiveBytesArc.lock().unwrap().to_owned());

    match std::fs::write(archiveName.clone(), fileBytes) {
        Err(_) => {
            eprintln!("Error: cannot create file `{}`", archiveName);
            std::process::exit(1);
        },
        Ok(_) => {}
    }
}

fn dfs(base: String, checked: std::sync::Arc<std::sync::Mutex<Vec<String>>>, currentIndex: usize,
    archiveBytes: std::sync::Arc<std::sync::Mutex<Vec<u8>>>, verbose: bool) -> (rsjson::Json, usize) {
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

            let fileBytes = match std::fs::read(stringPath) {
                Err(_) => Vec::new(),
                Ok(bytes) => bytes
            };

            if verbose {
                println!("Adding `{}/{}` file to archive", base, name.clone());
            }

            let fileSize = fileBytes.len();
            archiveBytes.lock().unwrap().extend(fileBytes);

            let mut indexList = Vec::<NodeContent>::new();
            indexList.push(NodeContent::Int(archiveIndex));

            indexList.push(NodeContent::Int(archiveIndex + fileSize));
            archiveIndex += fileSize;

            node.addNode(Node::new(
                name.clone(),
                NodeContent::List(indexList)
            ));


        } else if path.is_dir() {
            let (subContent, newIndex) = dfs(stringPath.clone(), checked.clone(), archiveIndex, archiveBytes.clone(), verbose);
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

/*fn walkJson(jsonNode: Json, parent: String, currentIndex: usize, archiveBytes: std::sync::Arc<std::sync::Mutex<Vec<u8>>>, verbose: bool) -> (rsjson::Json, usize) {
    let mut archiveIndex= currentIndex;
    let mut newJson = Json::new();

    for node in jsonNode.getAllNodes() {
        match node.getContent() {

            NodeContent::Null => {
                let fileBytes = match std::fs::read(format!("{}/{}", parent, node.getLabel())) {
                    Err(_) => Vec::new(),
                    Ok(bytes) => bytes
                };

                if verbose {
                    println!("Adding `{}/{}` file to archive", parent, node.getLabel());
                }

                let fileSize = fileBytes.len();
                archiveBytes.lock().unwrap().extend(fileBytes);

                let mut indexList = Vec::<NodeContent>::new();
                indexList.push(NodeContent::Int(archiveIndex));

                indexList.push(NodeContent::Int(archiveIndex + fileSize));
                archiveIndex += fileSize;

                newJson.addNode(Node::new(
                    node.getLabel(),
                    NodeContent::List(indexList)
                ));
            },

            NodeContent::Json(jnode) => {
                let (newSubNode, newIndex) = walkJson(jnode, node.getLabel(), archiveIndex, archiveBytes.clone(), verbose.clone());
                archiveIndex = newIndex;

                newJson.addNode(Node::new(
                    node.getLabel(),
                    NodeContent::Json(newSubNode)
                ));
            },

            _ => {}
        }
    }

    return (newJson, archiveIndex);
}*/

fn extractMap(archiveBytes: Vec<u8>) -> (usize, rsjson::Json){
    let mut jsonSizeBytes = Vec::new();
    let mut index: usize = 0;

    while index < archiveBytes.len() {
        let byte = archiveBytes.get(index).unwrap().to_owned();

        if byte == 123 {
            break
        }

        jsonSizeBytes.push(byte);
        index += 1;
    }

    let size = String::from_utf8(jsonSizeBytes).unwrap().parse::<usize>().unwrap();
    let jsonBytes = archiveBytes.get(index..index+size).unwrap().to_vec();
    return (index, rsjson::Json::fromString(String::from_utf8(jsonBytes).unwrap()).unwrap());
}

pub fn checkArchive(archivePath: String) {
    let archiveBytes = match std::fs::read(archivePath) {
        Err(_) => {
            eprintln!("Error: cannot read archive");
            std::process::exit(1);
        },

        Ok(bytes) => bytes
    };

    let (_, archiveMap) = extractMap(archiveBytes);
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
    let archiveBytes = match std::fs::read(archivePath) {
        Err(_) => {
            eprintln!("Error: cannot read archive");
            std::process::exit(1);
        },

        Ok(bytes) => bytes
    };

    /*let mut jsonSizeBytes = Vec::new();
    let mut index: usize = 0;

    while index < archiveBytes.len() {
        let byte = archiveBytes.get(index).unwrap().to_owned();

        if byte == 123 {
            break
        }

        jsonSizeBytes.push(byte);
        index += 1;
    }

    if verbose {
        println!("Extracting archive map");
    }

    let size = String::from_utf8(jsonSizeBytes).unwrap().parse::<usize>().unwrap();
    let jsonBytes = archiveBytes.get(index..index+size).unwrap().to_vec();
    let jsonMap = rsjson::Json::fromString(String::from_utf8(jsonBytes).unwrap()).unwrap();*/

    let (index, jsonMap) = extractMap(archiveBytes.clone());

    let archive = archiveBytes.get(index..archiveBytes.len()).unwrap().to_vec();
    let archiveMutex = std::sync::Mutex::new(archive);
    let archiveArc = std::sync::Arc::new(archiveMutex);

    extractAndCreate(extractionPath, jsonMap, archiveArc.clone(), verbose);
}

fn extractAndCreate(basePath: String, nodes: Json, archive: std::sync::Arc<std::sync::Mutex<Vec<u8>>>, verbose: bool) {
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

                let unlockedArchive = archive.lock().unwrap();
                let fileBytes = unlockedArchive.get(startByte..endByte).unwrap();

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