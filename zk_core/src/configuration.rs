use std::str::FromStr;

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

struct Config {
    working_dir: String, // TODO: use &str?
}

#[derive(Debug, PartialEq)]
enum ConfigParsingError {
    YamlBadFormat(String),
    YamlIsMultiDocument(),
    FieldMissing(String),
}

impl Config {
    fn load_yaml(s: &str) -> Result<Yaml, ConfigParsingError> {
        let mut configs: Vec<Yaml> = YamlLoader::load_from_str(s)
            .map_err(|err| ConfigParsingError::YamlBadFormat(err.to_string()))?;
        match configs.len() {
            1 => Ok(configs.swap_remove(0)),
            _ => Err(ConfigParsingError:: YamlIsMultiDocument())
        }
    }

    fn get_str_field(name: &'static str, config: &Yaml) 
        -> Result<String, ConfigParsingError>
    {
        config[name].as_str()
                    .map(|x| x.to_string())
                    .ok_or(ConfigParsingError::FieldMissing(name.to_string()))
    }
}

impl FromStr for Config {
    type Err = ConfigParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config = Config::load_yaml(s)?;
        Ok(Config {
            working_dir: Self::get_str_field("working_dir", &config)?,
        })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::{ConfigParsingError, Config};

    #[test]
    fn test_config_fromstr_correctly_parse_home_dir_with_path() {
        let config_content = "\nworking_dir: /home/user/notes\n";
        let config = Config::from_str(config_content).unwrap();
        assert_eq!(config.working_dir, "/home/user/notes");
    }

    #[test]
    fn test_config_fromstr_does_not_validate_paths() {
        let config_content = "working_dir: /";
        let config = Config::from_str(config_content).unwrap();
        assert_eq!(config.working_dir, "/");
    }

    #[test]
    fn test_config_fromstr_returns_error_if_working_dir_field_is_not_exist() {
        let config_content = "abs: a";
        let config = Config::from_str(config_content);
        assert_eq!(
            config.err(),
            Some(ConfigParsingError::FieldMissing("working_dir".to_string()))
        );
    }

    #[test]
    fn test_config_fromstr_returns_error_if_working_dir_field_has_invalid_type() {
        let config_content = "working_dir: [1, 2]";
        let config = Config::from_str(config_content);
        assert_eq!(
            config.err(),
            Some(ConfigParsingError::FieldMissing("working_dir".to_string()))
        );
    }

    #[test]
    fn test_config_fromstr_returns_error_if_yaml_invalid() {
        let config_content = "key: [1, 2]]\n";
        let config = Config::from_str(config_content);
        assert!(match config.err() {
            Some(ConfigParsingError::YamlBadFormat(_)) => true,
            _ => false
        });
    }

    #[test]
    fn test_config_fromstr_returns_error_if_yaml_is_multidocument() {
        let config_content = "a: b\n---\nworking_dir: a\n";
        let config = Config::from_str(config_content);
        assert!(match config.err() {
            Some(ConfigParsingError::YamlIsMultiDocument()) => true,
            _ => false
        });
    }
}
