use super::data_holder::*;

use std::{
    fs,
    ffi::OsString,
    error::Error,
    process,
    path::PathBuf,
    fs::File,
    io::Write,
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

pub fn read_temp_list_csv2(file_path: &PathBuf) -> Vec<f32> {
    let mut temp_list = Vec::<f32>::new();

    let mut rdr = 
    match csv::Reader::from_path(file_path) {
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading {:?} {}", &file_path, err);
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

pub fn read_part_csv(file_path: &PathBuf) -> (Part, Vec<(String, f32)>, Vec<(String, f32)> ) {
    let mut part = Part {
        name: "".to_string(),
        description: "".to_string(),
        temp: 0.0,
        absorbation_const: 0.0,
        cost_per_area: 0.0,
        has_ablator: false,
        height_min1: 0.0,
        height_min0: 0.0,
        height_max1: 0.0,
        height_max0: 0.0,
        areal_density_min: 0.0,
        areal_density_max: 0.0,
        tps_list_min: Vec::<(TPS, f32, Vec<DataTriplet>)>::new(),
        tps_list_max: Vec::<(TPS, f32, Vec<DataTriplet>)>::new(),
        data_min: Vec::<DataPair>::new(),
        data_max: Vec::<DataPair>::new(),
    };
    let mut read_max = false;

    let mut rdr = 
    match csv::ReaderBuilder::new().has_headers(false).from_path(&file_path) {
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading file {}\n,{}", &file_path.display(), err);
                            process::exit(1);}
    };
    let mut parts_min = Vec::<(String, f32)>::new();
    let mut parts_max = Vec::<(String, f32)>::new();

    for result in rdr.records() {
        let record = match result {
            Ok(result) => {result},
            Err(err) =>  {println!("Error while reading Structure.csv {}, {}", file_path.display() , err);
                            process::exit(1);}
        };
        match &record[0]{
            "Name" =>           part.name = record[1].to_string(),
            "Description" =>    part.description = record[1].to_string(),
            "Temperature" => {  part.temp = match record[1].parse::<f32>() {
                                    Ok(result) =>  result,
                                    Err(err) => {println!("{} Error while parsing Temperature to float", err);
                                                                process::exit(1);},
                                };
                            }
            "AbsorbationConstant" => {  part.absorbation_const = match record[1].parse::<f32>() {
                                            Ok(result) =>  result,
                                            Err(err) => {println!("{} Error while parsing AbsorbationConstant to float", err);
                                                                        process::exit(1);},
                                        };
                                    }
            "CostPerArea" => {  part.cost_per_area = match record[1].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing CostPerArea to float", err);
                                                                    process::exit(1);},
                                    };
                                }
            "HasAblator" => {  part.has_ablator = match record[1].parse::<bool>() {
                                    Ok(result) =>  result,
                                    Err(err) => {println!("{} Error while parsing HasAblator to bool", err);
                                                                process::exit(1);},
                                };
                            }
            "Min" =>        read_max = false,
            "Max" =>        read_max = true,
            "Structure" =>  {   if read_max {
                                    let name = record[1].to_string(); 
                                    let portion =  match record[2].parse::<f32>() {
                                            Ok(result) =>  result,
                                            Err(err) => {println!("{} Error while parsing Portion to float", err);
                                                                        process::exit(1);},
                                    };
                                    parts_max.push((name, portion))
                                } else {
                                    let name = record[1].to_string(); 
                                    let portion =  match record[2].parse::<f32>() {
                                            Ok(result) =>  result,
                                            Err(err) => {println!("{} Error while parsing Portion to float", err);
                                                                        process::exit(1);},
                                    };
                                    parts_min.push((name, portion))
                                }
                            }
            &_ => {}
        }
    }

    (part, parts_min, parts_max)
}


pub fn read_tps_csv(file_path: &PathBuf) -> TPS {

    let mut layer_top = Segment{..Default::default()};
    let mut layers = Vec::<Segment>::new();
    let mut structure = TPS{
        name: "".to_string(),
        temp_max: 0.0,
        data: Vec::<DataPair>::new(),
        areal_density: 0.0,
        tickness: 0.0,
        temp_list2: Vec::<f32>::new(),
        segments: Vec::<Segment>::new()
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
            "Temperature" => { structure.temp_max = match record[1].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Temperature to float", err);
                                            process::exit(1);},
                                    };
                             }
            "Top Layer" =>  {   layer_top.path = record[1].to_string();
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
                                layer_top.temp_hot_side = match record[4].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Temp Hot Side to float", err);
                                            process::exit(1);},
                                    };
                                layer_top.temp_cold_side = match record[5].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Temp Cold Side to float", err);
                                            process::exit(1);},
                                    };
                            },
            "Layer" =>       {  layers.push(Segment{..Default::default()});
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
                                layers.last_mut().unwrap().temp_hot_side = match record[4].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Temp Hot Side to float", err);
                                            process::exit(1);},
                                    };
                                layers.last_mut().unwrap().temp_cold_side = match record[5].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Temp Cold Side to float", err);
                                            process::exit(1);},
                                    };
                            },
            &_ => {}
        }
    }
    if structure.name == "" {
        {println!("Error Structure file lacks \"Name\" entry"); process::exit(1);}
    }
    if structure.temp_max == 0.0 {
        {println!("Error Structure file lacks \"Temperature\" entry"); process::exit(1);}
    }
    if layer_top.path == "" {
        {println!("Error Structure file lacks \"Top Layer\" entry"); process::exit(1);}
    }
    structure.segments.push(layer_top);
    structure.segments.append(&mut layers);
    structure
}

pub fn read_material_csv(segment: &mut Segment) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_path(&segment.path)?;
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
            segment.data_csv.push(DataPair{0: temp, 1: Data{cp, R_th: k, e: e}});
        } else {
            match &record[0]{
                "Name"              => segment.name = record[1].parse().unwrap(),
                "Temperature Limit" => match record[1].parse::<f32>() {
                                            Ok(result) => {segment.temp_max = result;},
                                            Err(_err) =>  {println!("{} Can not convert Temperature Limit into float", segment.name);
                                                                            process::exit(1);},
                                        },
                "Density"           => match record[1].parse::<f32>() {
                                            Ok(result) => {segment.density = result;},
                                            Err(_err) =>  {println!("{} Can not convert Density into float", segment.name);
                                                                            process::exit(1);},
                                        },
                "Temperature"       => found_temperature = true,
                &_                  => {},
            }
        }
    }
    segment.areal_density = segment.areal_density * segment.tickness;
    Ok(())
}

pub fn output_layer(layer: &Segment, path: &String, temp_list: &Vec<f32>) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;
    let output_file = path.clone() + "/" + &layer.name + "_csv_data.csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Layer", "Heat Capacity", "Thermal Insulance", "Emissivity",])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for (i, data) in layer.data_csv.iter().enumerate() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e ))?;
    }
    wtr.flush()?;


    let output_file = path.clone() + "/" + &layer.name + "_avg_r.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    /// TODO write this into seperate txt file along with csv
    //wtr.write_record(&["Name", &layer.name, "", "",""])?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity", "Temp Layer"])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for  data in layer.data_avg_r.iter() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;


    let output_file = path.clone() + "/" + &layer.name + "_height_adjusted.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    /// TODO write this into seperate txt file along with csv
    //wtr.write_record(&["Name", &layer.name, "", "",""])?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity", "Temp Layer"])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for  data in layer.data_height_adjust.iter() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;


    let output_file = path.clone() + "/" + &layer.name + "_tps_data.csv";
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
    for (i, data) in layer.data_tps_temp_map.iter().enumerate() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;

    let output_file = path.clone() + "/" + &layer.name + "_tps_temp_mult.csv";
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
    for (i, data) in layer.data_tps_temp_mult.iter().enumerate() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;


    Ok(())
}

pub fn output_tps(tps: &TPS, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;

    // write structure ino path
    let output_file = path.clone() + &tps.name + ".csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    /// TODO write this into seperate txt file along with csv
    //wtr.write_record(&["Name", "MaxTemp", "Height", "Mass/Area"])?;
    //wtr.serialize((structure.name.clone(), structure.temp, structure.tickness, structure.areal_density))?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity"])?;
    
    for data in tps.data.iter() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
    }
    wtr.flush()?;    

    // write layer into strucure Folder
    let directory = (&path).to_string() + &tps.name;
    fs::create_dir_all(&directory)?;
    for layer in tps.segments.clone() {
        output_layer(&layer,&directory, &tps.temp_list2)?;
    }


    let output_file = path.clone() + &tps.name + ".txt";
    wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };
    wtr.serialize(("Name", &tps.name))?;
    wtr.serialize(("Max Temp", tps.temp_max))?;
    wtr.serialize(("Areal Density", tps.areal_density))?;
    wtr.serialize(("Height Max", tps.tickness))?;
    wtr.serialize(("Layer", "Portion * Thickness"))?;
    for layer in &tps.segments {
        wtr.serialize((&layer.name, layer.tickness * layer.portion))?;    
    }
    wtr.flush()?;

    Ok(())
}

pub fn output_part(part: Part, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;
    let mut index = 0;

    let output_file = path.clone() + &part.name + "_min.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    /// TODO write this into seperate txt file along with csv
    // wtr.write_record(&["Name", "MaxTemp", "Height", "Mass/Area"])?;
    // wtr.serialize((part.name, part.temp, part.height_min.to_string() + " - " +  &part.height_max.to_string() , part.areal_density))?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity"])?;
    
    for (i, data) in part.data_min.iter().enumerate() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
        if data.0 >= part.temp - 25.0 && data.0 <= part.temp + 25.0 {
            index = i;
        }
    }
    wtr.flush()?;


    let output_file = path.clone() + &part.name + "_max.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    /// TODO write this into seperate txt file along with csv
    // wtr.write_record(&["Name", "MaxTemp", "Height", "Mass/Area"])?;
    // wtr.serialize((part.name, part.temp, part.height_min.to_string() + " - " +  &part.height_max.to_string() , part.areal_density))?;
    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity"])?;
    
    for data in part.data_max.iter() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
    }
    wtr.flush()?;


    let output_file = path.clone() + &part.name + ".cfg";
        
    let mut file = File::create(output_file)?;
    
    writeln!(file, "ROThermal_PRESET\n{{")?;
    writeln!(file, "    name = {}" , part.name)?;
    writeln!(file, "    description = {}" , part.description)?;
    writeln!(file, "    type = Skin\n")?;

    writeln!(file, "    skinMaxTemp = {}" , part.temp)?;
    writeln!(file, "    emissiveConstant = {}" , part.data_min[index].1.e)?;
    writeln!(file, "    absorptiveConstant = {}\n" , part.absorbation_const)?;

    writeln!(file, "    skinHeightMin = {:0.4}" , part.height_min1)?;
    writeln!(file, "    skinMassPerArea = {}" , part.areal_density_min)?;
    writeln!(file, "    skinSpecificHeatCapacity = {}" , part.data_min[index].1.cp)?;
    writeln!(file, "    thermalInsulance = {}\n" , f32::powf(part.data_min[index].1.R_th, -1.0))?;

    writeln!(file, "    skinHeightMax = {:0.4}" , part.height_max1)?;
    writeln!(file, "    skinMassPerAreaMax = {}" , part.areal_density_max)?;
    writeln!(file, "    skinSpecificHeatCapacityMax = {}" , part.data_max[index].1.cp)?;
    writeln!(file, "    thermalInsulanceMax = {}\n" , f32::powf(part.data_max[index].1.R_th, -1.0))?;

    writeln!(file, "    disableModAblator = {}" , part.has_ablator)?;
    writeln!(file, "    costPerArea = {}" , part.cost_per_area)?;
    writeln!(file, "}}")?;

    writeln!(file, "// Min: \n// Segment, Portion")?;
    for structure in &part.tps_list_min {
        writeln!(file, "// {}, {}",&structure.0.name, structure.1)?;    
    }
    writeln!(file, "\n// Max: \n// Segment, Portion")?;
    for structure in &part.tps_list_max {
        writeln!(file, "// {}, {}",&structure.0.name, structure.1)?;    
    }

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
