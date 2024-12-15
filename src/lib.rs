use rand::prelude::*;
use csv::Writer;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};



#[derive(Debug, Clone, PartialEq)]
pub struct Chromosome
{
    pub data:u64,
    pub fitness:f64,
    pub N:usize,
}

impl Chromosome
{
    pub fn new(N:usize)-> Self
    {
        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(0..1023);
        Chromosome {data:random_number, fitness:0.0 , N:N}
    }

    fn calculate_fitness(&mut self, data_sum:u64) -> f64
    {
        if let Some(diff) = data_sum.checked_sub(self.data){
        
            let fitness = ((20000 as i128 -  self.data as i128) as f64 - 0.52*diff as f64)*self.data as f64;
            if fitness < 0.0{
                return 0.0;
            }else{
                return fitness;
            }
        
        }else{
            return 0.0;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Run 
{
    Pcross:f32,
    Pmut:f32,
    L:u8,
    n:usize,
    z:u8,
    period:u32,
    population:Vec<Chromosome>,
    total_fitness:f64,
    data_sum:u64,
}

impl Run{
    pub fn new(Pcross:f32, Pmut:f32, L:u8, n:usize, z:u8)-> Self
    {
        let population:Vec<Chromosome> = (0..n).map(|_| Chromosome::new(n)).collect();
        Run{Pcross:Pcross, Pmut:Pmut, L:L, n:n, z:z, period:0, population:population, total_fitness:0.0, data_sum:0}
    }

    fn calculate_data_sum(&mut self)
    {
        self.data_sum = self.population.iter()
        .map(|x| x.data as u64)
        .sum::<u64>();
    }

    fn calculate_iteration_fitness(&mut self)->()
    {
        for ind in &mut self.population
        {
            let ind_fitness_old = ind.fitness;
            ind.fitness = ind.calculate_fitness(self.data_sum);
            self.total_fitness += ind.fitness - ind_fitness_old;
        }
    }

    fn assign_probability(&self, ind:&Chromosome)->f64
    {
        ind.fitness/self.total_fitness 
    }

    fn select(&self, probabilities:&Vec<f64>)->Chromosome
    {
        let rand_f:f64 = random();

        let mut cumulative_sum = 0.0;
        for i in 0..self.n{
            cumulative_sum += probabilities[i];
            if cumulative_sum >= rand_f
            {
                return self.population[i].clone();
            }
        }

        self.population[self.n - 1].clone()
    }

    fn recomb(&mut self)->()
    {
        let cumulative_probabilities:Vec<f64> = self.population.iter().map(|x| self.assign_probability(x)).collect(); 
        
        let next_gen:Vec<Chromosome> = (0..self.n).map(|_| self.select(&cumulative_probabilities)).collect();

        self.population = next_gen;
    }

    fn pairs(&self, mut old_population: Vec<Chromosome>, rng: &mut ThreadRng) -> Vec<(Chromosome, Chromosome)> {
        let mut pairs: Vec<(Chromosome, Chromosome)> = Vec::new();
        let mut paired_indices = vec![false; self.n]; // Track paired chromosomes by index

        for i in 0..self.n {
            if paired_indices[i] {
                continue; // Skip already paired chromosomes
            }

            let mut partner_idx = rng.gen_range(0..self.n);
            while paired_indices[partner_idx] || partner_idx == i {
                // Ensure partner is not already paired and not the same as current
                partner_idx = rng.gen_range(0..self.n);
            }

            // Mark both as paired
            paired_indices[i] = true;
            paired_indices[partner_idx] = true;

            // Push the pair
            pairs.push((
                old_population[i].clone(),
                old_population[partner_idx].clone(),
            ));
        }

        pairs
    }

    fn cross(&mut self) -> () {
        let mut thread_rng = rand::thread_rng();
        let mut old_population:Vec<Chromosome> = self.population.drain(..).collect();
        let pairs = self.pairs(old_population,&mut thread_rng);
        let mut new_population:Vec<Chromosome> = Vec::new();

        for pair in pairs.iter()
        {
            let mut clone1 = pair.0.clone();
            let mut clone2 = pair.1.clone();
                
            if thread_rng.gen::<f32>() < self.Pcross
            {
                let temp1 = (clone1.data << (self.L - self.z)) >> (self.L - self.z);
                let temp2 = (clone2.data << (self.L - self.z)) >> (self.L - self.z);
                
                for i  in 0..self.z     
                {
                    clone1.data &= !(1 << i);
                    clone2.data &= !(1 << i);
                }

                clone1.data |= temp2;
                clone2.data |= temp1;
                
            }
            new_population.push(clone1);
            new_population.push(clone2);
        }

        self.population = new_population;
    }                            

    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        for ind in &mut self.population {
            if rng.gen::<f32>() < self.Pmut {
                ind.data ^= 1 << rng.gen_range(0..self.L);
            }
        }
    }

    pub fn run(&mut self, iterations:u32)->(Vec<Chromosome>, Vec<(u64,f64)>)
    {

        let mut stats : Vec<(u64,f64)> = Vec::new();

        for _ in 0..iterations
        {
            self.calculate_data_sum();
            self.calculate_iteration_fitness();
            stats.push(self.iter_stats());
            self.recomb();
            self.cross();
            self.mutate();
        }

        (self.population.clone(), stats)
    }

    fn iter_stats(&self)->(u64,f64)
    {
        let sum = self.population.iter().map(|chromosome| chromosome.data).sum::<u64>();
        let mean = sum as f64 /  self.n as f64;

        let variance = self.population.iter()
            .map(|chromosome| (chromosome.data as f64 - mean).powi(2))
            .sum::<f64>() / self.n as f64;

        (sum, variance)
    }

}

pub fn save_iter_to_csv(data: &Vec<(u64, f64)>, file_name: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_name)?;
    let mut writer = csv::Writer::from_writer(file);

    
    writer.write_record(&["ind_out", "var"])?;

    
    for iter in data {
        writer.write_record(&[iter.0.to_string(), iter.1.to_string()])?;
    }


    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_chromosome() {
        let test = Chromosome::new(30);
        assert_eq!(test.data.count_ones() + test.data.count_zeros(), 64);
    }

    #[test]
    fn new_run(){
        let run = Run::new(0.2, 0.5, 32, 32, 16);
        assert!(run.Pcross == 0.2 && run.Pmut == 0.5 && run.L == 32 && run.n == 32 && run.z == 16 && run.period == 0 && run.population.len() == 32);
    }

    #[test]
    fn select_test(){
        let test_run = Run::new(0.2, 0.5, 32, 32, 16);
        let mut probabilities:Vec<f64> = (0..32).map(|_| random()).collect();
        let sum:f64 = probabilities.iter().sum();
        probabilities.iter_mut().for_each(|x| *x /= sum);
        let new_var = test_run.select(&probabilities);
        assert!(1==1);
    }

    #[test]
    fn recomb_test(){
        let mut test_run = Run::new(0.2, 0.5, 32, 32, 16);
        test_run.recomb();

        assert_eq!(test_run.population.len(), 32);
    }

    #[test]
    fn shift_test(){
        let mut number = 0b0000_1100;
        let mut shift = 14;
        let n = 3;
        for i in (0..n)
        {
            number &= !(1 << i);
        }
        shift = (shift << (8 - n)) >> (8 - n);
        number |= shift;
        assert_eq!(number, 14)
    }

    #[test]
    fn flip_test()
    {
        let mut ind:u8 = 0b1000_0000;
        ind ^= 1 << 7;
        assert_eq!(ind, 0);
    }

    #[test]
    fn cross_test(){
        let mut test_run = Run::new(0.2, 0.5, 64, 32, 16);
        let old_population = test_run.population.clone();
        test_run.cross();
        assert!((test_run.population != old_population)&&(old_population.len() == test_run.population.len()))
    }

    #[test]
    fn run_test()
    {
        let mut test_run = Run::new(0.322, 0.00322, 10, 30, 2);

        let mut old_population = test_run.population.clone();

        let result = test_run.run(10000).0;
        old_population.sort_by(|a, b| a.data.cmp(&b.data));
        let mut sorted_result = result.clone();
        sorted_result.sort_by(|a, b| a.data.cmp(&b.data));
        
        let sum = result.iter().fold(0.0, |a,b| a + b.fitness);

        for (idx, ind) in result.iter().enumerate() {
            println!("idx: {}",ind.fitness/sum);
        }
        let ind_out = result.iter().fold(0, |a,b| a + b.data);
        println!("industry_output: {:?}, avg_out_per_ind:{:?},  industry_util: {:?}", ind_out, ind_out as f64/test_run.n as f64 ,sum);
        println!("best ind: {:?}", result.iter().max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap()));
        assert!((result != old_population)&&(result.len() == old_population.len()))
    }

}
