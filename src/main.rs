mod git;
mod ripgrep;

struct HandleExercises {
    exercises_availables: Vec<(
        String,
        Box<dyn FnMut(&[String]) -> Result<(), Box<dyn std::error::Error>>>,
    )>,
}

impl HandleExercises {
    pub fn new() -> Self {
        let mut exercises = vec![];

        exercises.push((
            "ripgrep".into(),
            Box::new(ripgrep::run)
                as Box<dyn FnMut(&[String]) -> Result<(), Box<dyn std::error::Error>>>,
        ));

        exercises.push((
            "git".into(),
            Box::new(git::run)
                as Box<dyn FnMut(&[String]) -> Result<(), Box<dyn std::error::Error>>>,
        ));

        exercises.push((
            "testing".into(),
            Box::new(|args: &[String]| Ok(println!("Testing: {:?}", args)))
                as Box<dyn FnMut(&[String]) -> Result<(), Box<dyn std::error::Error>>>,
        ));

        exercises.push((
            "testing_error".into(),
            Box::new(|args: &[String]| Err(format!("Testing: {:?}", args).into()))
                as Box<dyn FnMut(&[String]) -> Result<(), Box<dyn std::error::Error>>>,
        ));

        Self {
            exercises_availables: exercises,
        }
    }

    pub fn text_exercises(&self) -> String {
        self.exercises_availables
            .iter()
            .map(|x| format!("  - {}\n", x.0))
            .collect()
    }

    pub fn error_exercises(&self) -> Box<dyn std::error::Error> {
        format!(
            r#"Please add exercise to be run, example:
cargo run -- <name_exercise>

Valid exercises:
{}"#,
            self.text_exercises()
        )
        .into()
    }

    pub fn run_exercise(
        &mut self,
        exercise: &str,
        args: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (name, func) in self.exercises_availables.iter_mut() {
            if exercise == name {
                return func(args);
            }
        }

        Err(self.error_exercises())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args();
    run(args)
}

fn run<I>(args: I) -> Result<(), Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    let mut handler = HandleExercises::new();

    let valid_args = match args.get(1..) {
        Some(v) if v.len() > 0 => v,
        _ => {
            return Err(handler.error_exercises());
        }
    };

    let selected_exer: &str = valid_args.get(0).ok_or(handler.error_exercises())?;
    let missing_args: &[_] = valid_args.get(1..).unwrap();

    handler.run_exercise(selected_exer, missing_args)?;

    Ok(())
}

#[cfg(test)]
mod test_main {
    use super::*;

    #[test]
    pub fn no_valid_exercise() {
        // Empty exercise
        let args: Vec<String> = vec!["./src/main.rs".into()];
        let result = run(args).unwrap_err();
        assert!(result.to_string().contains("Please add exercise"));

        // Invalid Exercise
        let args: Vec<String> = vec!["./src/main.rs".into(), "invalid_exercise".into()];
        let result = run(args).unwrap_err();
        assert!(result.to_string().contains("Please add exercise"));
    }

    #[test]
    pub fn valid_exercise_propagate_error() {
        // propagate errors
        let args: Vec<String> = vec!["./src/main.rs".into(), "testing_error".into()];
        let result = run(args).unwrap_err().to_string();
        assert_eq!(result, "Testing: []");
    }
}
