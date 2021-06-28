pub struct AdventYear {
    year: u16,
    advents: Vec<Box<dyn Advent>>,
}

impl AdventYear {
    pub fn new(year: u16, advents: Vec<Box<dyn Advent>>) -> Self {
        Self { year, advents }
    }

    pub fn get_year(&self) -> u16 {
        self.year
    }

    pub fn into_advents(self) -> Vec<Box<dyn Advent>> {
        self.advents
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Advent>> {
        self.advents.iter()
    }
}

pub trait Advent {
    fn get_index(&self) -> u8;

    fn skip(&self) -> bool {
        false
    }

    fn get_input_names(&self) -> Vec<String> {
        vec!["input.txt".to_owned()]
    }

    /// Process the given data. The data is the content of the files provided by
    /// `Advent::get_input_names`
    fn process_input(&self, data: Vec<String>);
}

pub struct SkippedAdvent(u8);

impl SkippedAdvent {
    pub fn new(advent: u8) -> Self {
        Self(advent)
    }
}

impl Advent for SkippedAdvent {
    fn get_index(&self) -> u8 {
        self.0
    }

    fn skip(&self) -> bool {
        true
    }

    fn get_input_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn process_input(&self, _data: Vec<String>) {
        unimplemented!()
    }
}
