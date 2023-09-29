use super::data_holder::*;

use std::{
    fs,
    ffi::OsString,
    error::Error,
    process,
    path::PathBuf,
};


pub fn get_files (path: String, extension: OsString) -> Vec<PathBuf> {
    let paths = fs::read_dir(path)
    .unwrap()
    .filter_map(|e| e.ok())
    .map(|e| e.path())
    .filter(|path| path.extension().map_or(false, |ext| ext == extension))
    .collect::<Vec<PathBuf>>();

    paths
}

pub fn read_temp_list_csv(file_path: &PathBuf) -> Vec<i32> {
    let mut temp_list = Vec::<i32>::new();

    let mut rdr = 
    match csv::Reader::from_path(file_path) {
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    
    for result in rdr.records() {
        let record = match result{
            Ok(result) => {result},
            Err(err) =>  {println!("Error while handling Results.csv {}", err);
                            process::exit(1);}
        };

        match record[0].parse::<i32>() {
            Ok(result) => {temp_list.push(result)},
            Err(err) =>  {println!("Error while processing Results.csv Temperatures {}", err);
                            process::exit(1);}
        };
    }
    temp_list
}

pub fn read_temp_list_csv2(file_path: &PathBuf) -> Vec<f32> {
    let mut temp_list = Vec::<f32>::new();

    let mut rdr = 
    match csv::Reader::from_path(file_path) {
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    
    for result in rdr.records() {
        let record = match result{
            Ok(result) => {result},
            Err(err) =>  {println!("Error while handling Results.csv {}", err);
                            process::exit(1);}
        };

        match record[0].parse::<f32>() {
            Ok(result) => {temp_list.push(result)},
            Err(err) =>  {println!("Error while processing Results.csv Temperatures {}", err);
                            process::exit(1);}
        };
    }
    temp_list
}

pub fn read_part_csv(file_path: &PathBuf) -> (Construction, Vec<(String, f32)> ) {
    let mut part = Construction {
        name: "".to_string(),
        temp: 0.0,
        height_max: 0.0,
        height_min: 0.0,
        areal_density: 0.0,
        structures: Vec::<(Structure, f32)>::new(),
        data: Vec::<DataPair>::new(),
    };

    let mut rdr = 
    match csv::ReaderBuilder::new().has_headers(false).from_path(&file_path) {
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading file {}\n,{}", &file_path.display(), err);
                            process::exit(1);}
    };
    let mut parts = Vec::<(String, f32)>::new();

    for result in rdr.records() {
        let record = match result {
            Ok(result) => {result},
            Err(err) =>  {println!("Error while reading Structure.csv {}, {}", file_path.display() , err);
                            process::exit(1);}
        };
        match &record[0]{
            "Name" =>           part.name = record[1].to_string(),
            "Temperature" => {  part.temp = match record[1].parse::<f32>() {
                                    Ok(result) =>  result,
                                    Err(err) => {println!("{} Error while parsing Temperature to float", err);
                                                                process::exit(1);},
                                };
                            }
            "Structure" =>  {   let name = record[1].to_string(); 
                                let portion =  match record[2].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Portion to float", err);
                                                                    process::exit(1);},
                                };
                                parts.push((name, portion))
                            }
            &_ => {}
        }
    }

    (part, parts)
}


pub fn read_structure_csv(file_path: &PathBuf) -> Structure {

    let mut layer_top = Layer{..Default::default()};
    let mut layers = Vec::<Layer>::new();
    let mut structure = Structure{
        name: "".to_string(),
        temp: 0.0,
        data: Vec::<DataPair>::new(),
        areal_density: 0.0,
        tickness: 0.0,
        temp_list2: Vec::<f32>::new(),
        layers: Vec::<Layer>::new()
    };
    let mut rdr = 
    match csv::ReaderBuilder::new().has_headers(false).from_path(&file_path) {
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading file {}\n,{}", &file_path.display(), err);
                            process::exit(1);}
    };

    for result in rdr.records() {
        let record = match result {
            Ok(result) => {result},
            Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
        };
        match &record[0] {
            "Name" => structure.name = record[1].to_string(),
            "Temperature" => { structure.temp = match record[1].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Temperature to float", err);
                                            process::exit(1);},
                                    };
                             } 
            "Top Layer" =>  { layer_top.path = record[1].to_string();
                              layer_top.portion = match record[2].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Top Portion to float", err);
                                            process::exit(1);},
                                    };
                              layer_top.tickness = match record[3].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Top Tickness to float", err);
                                            process::exit(1);},
                                    };
                            },
            "Layer" =>       { layers.push(Layer{..Default::default()});
                               layers.last_mut().unwrap().path = record[1].to_string();
                               layers.last_mut().unwrap().portion = match record[2].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Portion to float", err);
                                            process::exit(1);},
                                    };
                               layers.last_mut().unwrap().tickness = match record[3].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Tickness to float", err);
                                            process::exit(1);},
                                    };
                            },
            &_ => {}
        }
    }
    if structure.name == "" {
        {println!("Error Structure file lacks \"Name\" entry"); process::exit(1);}
    }
    if structure.temp == 0.0 {
        {println!("Error Structure file lacks \"Temperature\" entry"); process::exit(1);}
    }
    if layer_top.path == "" {
        {println!("Error Structure file lacks \"Top Layer\" entry"); process::exit(1);}
    }
    structure.layers.push(layer_top);
    structure.layers.append(&mut layers);
    structure
}

pub fn read_material_csv(layer: &mut Layer) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_path(&layer.path)?;
    let mut found_temperature: bool = false;

    for result in rdr.records() {
        let record = result?;

        if found_temperature == true {
            let temp;
            let cp;
            let k; 
            let e; 
            match record[0].parse::<f32>() {
                Ok(result) => {temp = result},
                Err(_err) =>  temp = 0.0,
            };
            match record[1].parse::<f32>() {
                Ok(result) => {cp = result},
                Err(_err) =>  cp = 0.0,
            };
            match record[2].parse::<f32>() {
                Ok(result) => {k = result},
                Err(_err) =>  k = 0.0,
            };
            match record[3].parse::<f32>() {
                Ok(result) => {e = result},
                Err(_err) =>  e = 0.0,
            };
            layer.thermal_prop_layer_temp.push(DataPair{0: temp, 1: Data{cp, R_th: k, e: e}});
        } else if record[0] == *"Temperature Limit" {
            match record[1].parse::<f32>() {
                Ok(result) => {layer.temp_max = result;},
                Err(_err) =>  {println!("{} Can not convert Temperature Limit into float", layer.name);
                                process::exit(1);},
            };
        } else if record[0] == *"Density" {
            match record[1].parse::<f32>() {
                Ok(result) => {layer.density = result;},
                Err(_err) =>  {println!("{} Can not convert Density into float", layer.name);
                                process::exit(1);},
            };
        } else if record[0] == *"Temperature" {
            found_temperature = true;
        } else if  record[0] == *"Name" {
            layer.name = record[1].parse().unwrap();
        }
    }

    let path = "out/test/layers/";
    fs::create_dir_all(&path)?;

    let output_file = path.to_owned() + &layer.name + ".csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    /// TODO write this into seperate txt file along with csv
    // wtr.write_record(&["Name", "MaxTemp", "Height", "Mass/Area"])?;
    // wtr.serialize((part.name, part.temp, part.height_min.to_string() + " - " +  &part.height_max.to_string() , part.areal_density))?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity"])?;
    
    for data in layer.thermal_prop_layer_temp.iter() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
    }
    wtr.flush()?;

    Ok(())
}

pub fn output_layer(layer: &Layer, path: &String, temp_list: &Vec<f32>) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;
    let output_file = path.clone() + "/" + &layer.name + "_layer_temp.csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    /// TODO write this into seperate txt file along with csv
    //wtr.write_record(&["Name", &layer.name, "", "",""])?;
    wtr.write_record(&["Temp Layer", "Heat Capacity", "Thermal Insulance", "Emissivity",])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for (i, data) in layer.thermal_prop_layer_temp.iter().enumerate() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e ))?;
    }
    wtr.flush()?;


    let output_file = path.clone() + "/" + &layer.name + "_struct_temp.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    /// TODO write this into seperate txt file along with csv
    //wtr.write_record(&["Name", &layer.name, "", "",""])?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity","Temp Layer"])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for (i, data) in layer.thermal_prop_struct_temp.iter().enumerate() {
        if i < temp_list.len() {
            wtr.serialize((temp_list[i], data.1.cp, data.1.R_th, data.1.e, data.0))?;
        } else {
            wtr.serialize(("index out of bounds", data.1.cp, data.1.R_th, data.1.e, data.0))?;
        }
    }
    wtr.flush()?;


    let output_file = path.clone() + "/" + &layer.name + "_struct_temp_frac.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    /// TODO write this into seperate txt file along with csv
    //wtr.write_record(&["Name", &layer.name, "", "",""])?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity","Temp Layer"])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for (i, data) in layer.thermal_prop_struct_temp_frac.iter().enumerate() {
        wtr.serialize((temp_list[i], data.1.cp, data.1.R_th, data.1.e, data.0))?;
    }
    wtr.flush()?;

    Ok(())
}

pub fn output_structure(structure: &Structure, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;

    // write structure ino path
    let output_file = path.clone() + &structure.name +"_" + &format!("{:.2}",structure.tickness * 1000.0) + ".csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    /// TODO write this into seperate txt file along with csv
    //wtr.write_record(&["Name", "MaxTemp", "Height", "Mass/Area"])?;
    //wtr.serialize((structure.name.clone(), structure.temp, structure.tickness, structure.areal_density))?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity"])?;
    
    for data in structure.data.iter() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
    }
    wtr.flush()?;    

    // write layer into strucure Folder
    let directory = (&path).to_string() + &structure.name +"_" + &format!("{:.2}",structure.tickness * 1000.0);
    fs::create_dir_all(&directory)?;
    for layer in structure.layers.clone() {
        output_layer(&layer,&directory, &structure.temp_list2)?;
    }


    let output_file = path.clone() + &structure.name +"_" + &format!("{:.2}",structure.tickness * 1000.0) + ".txt";
    wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    wtr.serialize(("Name", &structure.name,""))?;
    wtr.serialize(("Max Temp", structure.temp,""))?;
    wtr.serialize(("Areal Density", structure.areal_density,""))?;
    wtr.serialize(("Height Max", structure.tickness,""))?;
    for layer in &structure.layers {
        wtr.serialize(("layer", &layer.name, layer.tickness * layer.portion))?;    
    }
    wtr.flush()?;

    Ok(())
}

pub fn output_part(part: Construction, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;

    let output_file = path.clone() + &part.name +"_" + &format!("{:.2}", part.height_max * 1000.0)+ ".csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    /// TODO write this into seperate txt file along with csv
    // wtr.write_record(&["Name", "MaxTemp", "Height", "Mass/Area"])?;
    // wtr.serialize((part.name, part.temp, part.height_min.to_string() + " - " +  &part.height_max.to_string() , part.areal_density))?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity"])?;
    
    for data in part.data.iter() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
    }
    wtr.flush()?;

    let output_file = path.clone() + &part.name +"_" + &format!("{:.2}", part.height_max * 1000.0)+ ".txt";

    wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    wtr.serialize(("Name", part.name,""))?;
    wtr.serialize(("Max Temp", part.temp,""))?;
    wtr.serialize(("Areal Density", part.areal_density,""))?;
    wtr.serialize(("Height Max", part.height_max,""))?;
    wtr.serialize(("Height Min", part.height_min,""))?;
    for structure in &part.structures {
        wtr.serialize(("layer", &structure.0.name, structure.1))?;    
    }
    wtr.flush()?;

    Ok(())
}

pub fn output_data_Triplet(name: &String, data: &Vec<DataTriplet>, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;

    let output_file = path.clone() + "/" + &name + ".csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    /// TODO write this into seperate txt file along with csv
    // wtr.write_record(&["Name", "MaxTemp", "Height", "Mass/Area"])?;
    // wtr.serialize((part.name, part.temp, part.height_min.to_string() + " - " +  &part.height_max.to_string() , part.areal_density))?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity", "Temp Layer"])?;
    
    for data in data.iter() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn output_data_Pair(name: &String, data: &Vec<DataPair>, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;

    let output_file = path.clone() + "/" + &name + ".csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    /// TODO write this into seperate txt file along with csv
    // wtr.write_record(&["Name", "MaxTemp", "Height", "Mass/Area"])?;
    // wtr.serialize((part.name, part.temp, part.height_min.to_string() + " - " +  &part.height_max.to_string() , part.areal_density))?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity"])?;
    
    for data in data {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
    }
    wtr.flush()?;
    Ok(())
}