use crate::advents::Advent;

pub trait AdventState {
    const INPUT_FILES: &'static [&'static str];

    fn new(input_file: &'static str, input_content: String) -> Self;

    fn run(self);
}

pub struct StatefulAdvent<T: AdventState> {
    index: u8,
    _t: std::marker::PhantomData<*const T>,
}

impl<T: AdventState> StatefulAdvent<T> {
    pub fn new(index: u8) -> Self {
        Self {
            index,
            _t: std::marker::PhantomData,
        }
    }
}

impl<T: AdventState> Advent for StatefulAdvent<T> {
    fn get_index(&self) -> u8 {
        self.index
    }

    fn get_input_names(&self) -> Vec<String> {
        T::INPUT_FILES.iter().copied().map(String::from).collect()
    }

    fn process_input(&self, data: Vec<String>) {
        data.into_iter()
            .zip(T::INPUT_FILES.iter().copied())
            .for_each(|(input, file_name)| {
                println!("\nProcessing file {}", file_name);
                T::new(file_name, input).run();
            })
    }
}
