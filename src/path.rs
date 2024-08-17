use std::path;

#[derive(Debug, Clone)]
pub(crate) struct Path {
    directories: Vec<String>
}

impl Path {
    pub(crate) fn parse(input: &str) -> Result<Path, anyhow::Error> {
        Ok(Path {
            directories: input.split(":").map(|p| p.to_string()).collect()
        })
    }

    pub(crate) fn find_command(&self, command_name: &str) -> Option<String> {
        let mut found_command: Option<String> = None;
        let mut iterator = self.directories.iter();
        let mut next_directory = iterator.next();
        while next_directory.is_some() && found_command.is_none() {
            if let Some(directory) = next_directory {
                let path_to_command = path::Path::new(directory).join(command_name);
                if path_to_command.is_file() {
                    found_command = path_to_command.to_str().map(|x| x.to_string());
                }
            }
            next_directory = iterator.next();
        }
        found_command
    }
}