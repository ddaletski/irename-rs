use std::path::{Component, Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut normalized = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    let mut is_absolute = false;
    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                normalized.push(component.as_os_str());
                is_absolute = true;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(c) => {
                normalized.push(c);
            }
        }
    }

    if is_absolute {
        normalized
    } else {
        std::env::current_dir().unwrap().join(normalized)
    }
}

pub fn split_path(mut path: PathBuf) -> Option<(PathBuf, String)> {
    if let Some(name) = path.file_name().map(|s| s.to_owned()) {
        path.pop();
        Some((path, name.to_str().unwrap().to_owned()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use proptest::proptest;

    use super::*;

    mod split_path {
        use super::*;

        proptest! {
            #[test]
            fn splittable(dir_str in "([.]{0,2}/)?((([0-9a-zA-Z_]+)|([.]{1,2}))/)*",
                          expected_filename in "[0-9a-zA-Z_]+") {

                let expected_dir = PathBuf::from(dir_str);
                let src_path = expected_dir.join(PathBuf::from(expected_filename.clone()));

                let (dir, filename) = split_path(src_path.clone())
                    .expect(&format!("can't split path: {:?}", src_path));

                prop_assert_eq!(dir, expected_dir);
                prop_assert_eq!(filename, expected_filename);
            }

            #[test]
            fn unsplittable(path in "([.]{0,2})/?") {
                let expected_dir = PathBuf::from(path);
                prop_assert_eq!(split_path(expected_dir.clone()), None);

            }
        }
    }
}
