use std::collections::HashMap;

#[derive(Clone)]  
pub struct AdressManager {
    adrbox: AdressBox,

    // memory range
    mem_range_start: u64,
    mem_range_end: u64,
    mem_range: bool,
}

impl AdressManager {
    pub fn new(mem_range: (u64, u64)) -> Self {
        let (mem_range_start, mem_range_end) = mem_range;
        let mut mem_rang = true;
        if mem_range_start == mem_range_end { mem_rang = false; } // no mem range

        Self {
            adrbox: AdressBox::new(mem_range_start),
            mem_range: mem_rang,
            mem_range_start: mem_range_start,
            mem_range_end: mem_range_end,
        }
    }

    pub fn reg_var(&mut self, name: &String, size: u64) -> i128 {
        let adr = self.adrbox.last_adr.clone();

        let entry = AdressBoxEntry {
            size: size,
            adr: adr,
        };

        if self.adrbox.adress.contains_key(name) { // 
            return -1;
        }

        self.adrbox.adress.insert(name.into(), entry.clone());
        self.adrbox.last_adr += size;

        adr as i128
    } 

    pub fn get_entry(&self, name: &String) -> &AdressBoxEntry {
        match self.adrbox.adress.get(name) {
            Some(entry) => entry,
            None => &AdressBoxEntry { size: 0, adr: 0},
        }
    }

    pub fn get_adr(&self, name: &String) -> i128 {
        let entry = self.get_entry(name).to_owned();

        if entry.size == 0 { 
            return -1; 
        }

        entry.adr as i128
    }

    pub fn get_size(&self, name: &String) -> i128 {
        let entry = self.get_entry(name).to_owned();

        if entry.size == 0 { 
            return -1; 
        }

        entry.size as i128
    }
}

#[derive(Clone)]

pub struct AdressBox {
    pub adress: HashMap<String, AdressBoxEntry>,
    pub last_adr: u64,
}

impl AdressBox {
    pub fn new(start_adr: u64) -> Self {
        Self {
            last_adr: start_adr,
            adress: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct AdressBoxEntry {
    pub size: u64,
    pub adr: u64,
}