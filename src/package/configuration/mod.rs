use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{collections::hash_map::HashMap, io::BufRead};

use anyhow::{anyhow, Context, Ok, Result};
use shlex;

const MANIFEST: &str = "MANIFEST";

pub trait Hook {
    // TODO
}

pub trait FileInstaller {
    // TODO
}

pub struct Configuration {
    root: PathBuf,
    subdirs: HashMap<String, Configuration>,
    pre_hooks: Vec<Box<dyn Hook>>,
    post_hooks: Vec<Box<dyn Hook>>,
    files: Vec<Box<dyn FileInstaller>>,
}

pub trait Parser {
    fn parse(configuration: &mut Configuration, args: &Vec<String>) -> Result<()>;
}

impl Configuration {
    pub fn new(root: PathBuf) -> Result<Self> {
        let mut conf = Configuration {
            root: root.clone(),
            subdirs: HashMap::new(),
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
            files: Vec::new(),
        };
        let manifest = root.join(MANIFEST);
        conf.parse(&manifest)
            .with_context(|| format!("failed to load {}", manifest.display()))?;
        return Ok(conf);
    }
    fn parse_line(&mut self, line_num: usize, line: &str) -> Result<()> {
        if line.is_empty() || line.starts_with("#") {
            return Ok(());
        }
        let args = shlex::split(&line).ok_or(anyhow!("failed to split line {:?}", line))?;
        println!("{}/{}:{} {:?}", self.root.display(), MANIFEST, line_num, args);
        Ok(())
    }
    fn parse(&mut self, manifest_path: &PathBuf) -> Result<()> {
        let manifest = File::open(manifest_path)
            .with_context(|| format!("failed to open {}", manifest_path.display()))?;
        let reader = BufReader::new(manifest);
        for (line_idx, line_or) in reader.lines().enumerate() {
            let line_num = line_idx + 1;
            let line = line_or.with_context(|| {
                format!(
                    "failed to read line {} from {}",
                    line_num,
                    manifest_path.display()
                )
            })?;
            self.parse_line(line_num, &line).with_context(|| {
                format!(
                    "failed to parse line {} {:?} from {}",
                    line_num,
                    line,
                    manifest_path.display()
                )
            })?;
        }
        Ok(())
    }
}
