use super::data_holder::*;

use csv::StringRecord;
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

pub fn read_part_csv(file_path: &PathBuf) -> (Part, Vec<(String, f32, f32, f32)> ) {
    let mut part = Part {
        name: "".to_string(),
        description: "".to_string(),
        temp: 0.0,
        absorbation_const: 0.0,
        cost_per_area: 0.0,
        has_ablator: false,
        height_min: 0.0,
        height_max: 0.0,
        areal_density_min: 0.0,
        areal_density_max: 0.0,
        tps_list: Vec::<(TPS, f32, Vec<DataTriplet>, Vec<DataTriplet>)>::new(),
        data_min: Vec::<DataPair>::new(),
        data_max: Vec::<DataPair>::new(),
    };

    let mut rdr = 
    match csv::ReaderBuilder::new().has_headers(false).from_path(&file_path) {
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading file {}\n,{}", &file_path.display(), err);
                            process::exit(1);}
    };
    let mut parts = Vec::<(String, f32, f32, f32)>::new();

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
            "Structure" =>  {
                                let name = record[1].to_string(); 
                                let portion =  match record[2].parse::<f32>() {
                                        Ok(result) =>  result,
                                        Err(err) => {println!("{} Error while parsing Portion to float", err);
                                                                    process::exit(1);},
                                };
                                let height_min =  match &record[3] {
                                    "max" => f32::INFINITY,
                                    "min" => f32::NEG_INFINITY,
                                    &_ =>   match record[3].parse::<f32>() {
                                                Ok(result) =>  result,
                                                Err(err) => {println!("{} Error while parsing Height Min to float", err);
                                                                            process::exit(1);},
                                            },
                                };
                                let height_max =  match &record[4] {
                                    "max" => f32::INFINITY,
                                    "min" => f32::NEG_INFINITY,
                                    &_ =>   match record[4].parse::<f32>() {
                                                Ok(result) =>  result,
                                                Err(err) => {println!("{} Error while parsing Height Max to float", err);
                                                                            process::exit(1);},
                                            },
                                };
                                parts.push((name, portion, height_min, height_max))
                            }
            &_ => {}
        }
    }
    (part, parts)
}


pub fn read_tps_csv(file_path: &PathBuf) -> TPS {
    let mut read_max = false;
    let mut layers_min = Vec::<Segment>::new();
    let mut layers_max = Vec::<Segment>::new();
    let mut structure = TPS {..Default::default()};
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
            "Min" =>        read_max = false,
            "Max" =>        read_max = true,
            "Top Layer" =>  {   if read_max {
                                    structure.segments_max.push(read_segment(&record));
                                } else {
                                    structure.segments_min.push(read_segment(&record));
                                }
                            },
            "Layer" =>      {  if read_max {
                                    layers_max.push(read_segment(&record));
                                } else {
                                    layers_min.push(read_segment(&record));
                                }

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
    structure.segments_min.append(&mut layers_min);
    if !layers_max.is_empty() {
        structure.segments_max.append(&mut layers_max);
    } 
    structure
}

fn read_segment(record: &StringRecord) -> Segment {
    let mut segment = Segment{..Default::default()};
    segment.path = record[1].to_string();
    segment.portion = match record[2].parse::<f32>() {
            Ok(result) =>  result,
            Err(err) => {println!("{} Error while parsing Top Portion to float", err);
                process::exit(1);},
        };
    segment.tickness = match record[3].parse::<f32>() {
            Ok(result) =>  result,
            Err(err) => {println!("{} Error while parsing Top Tickness to float", err);
                process::exit(1);},
        };
    segment.temp_hot_side = match record[4].parse::<f32>() {
            Ok(result) =>  result,
            Err(err) => {println!("{} Error while parsing Temp Hot Side to float", err);
                process::exit(1);},
        };
    segment.temp_cold_side = match record[5].parse::<f32>() {
            Ok(result) =>  result,
            Err(err) => {println!("{} Error while parsing Temp Cold Side to float", err);
                process::exit(1);},
        };
    segment
}

pub fn read_material_csv(segment: &mut Segment) -> Result<(), Box<dyn Error>> {
    let mut rdr = match csv::ReaderBuilder::new().has_headers(false).from_path(&segment.path) {
                            Ok(result) => {result},
                            Err(_err) =>  {println!("Error opening & reading file {} ", &segment.path);
                                                    process::exit(1);},
    };
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
                "Density"           => {    match record[1].parse::<f32>() {
                                                Ok(result) => {segment.density = result;},
                                                Err(_err) =>  {println!("{} Can not convert Density into float", segment.name);
                                                                                process::exit(1);},
                                            };
                                            match record[3].parse::<f32>() {
                                                Ok(result) => {segment.additive_areal_weight = result;},
                                                Err(_err) =>  {if &record[2] == "Additive Areal Weight" {
                                                                                    println!("{} Can not convert Additive Areal Weight into float", segment.name);
                                                                                    process::exit(1);}}}
                                        },
                "Temperature"       => found_temperature = true,
                &_                  => {},
            }
        }
    }
    //segment.areal_density = segment.areal_density * segment.tickness + segment.additive_areal_weight;
    Ok(())
}

pub fn output_layer(layer: &Segment, path: &String) -> Result<(), Box<dyn Error>> {
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
    for data in layer.data_csv.iter() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e ))?;
    }
    wtr.flush()?;


    let output_file = path.clone() + "/" + &layer.name + "_avg_r.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

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
    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity", "Temp Layer"])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for  data in layer.data_height_adjust.iter() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;


    let output_file = path.clone() + "/" + &layer.name + "_data_tps_temp_map.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity","Temp Layer"])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for data in layer.data_tps_temp_map.iter() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;

    let output_file = path.clone() + "/" + &layer.name + "_tps_temp_mult.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity","Temp Layer"])?;
    
    //    thermal_prop_layer_temp
    //    thermal_prop_struct_temp
    //    thermal_prop_struct_temp_frac
    for data in layer.data_tps_temp_mult.iter() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;


    Ok(())
}

pub fn output_tps(tps: &TPS, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;
    fs::create_dir_all(path.to_owned() + "csv")?;
    let mut index = 0;
    // write structure into path
    let output_file = path.clone() + "csv/" + &tps.name + "_min.csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity"])?;
    
    for (i, data) in tps.data_min.iter().enumerate() {
        wtr.serialize((data.0, data.1.cp, 1.0 / data.1.R_th, data.1.e))?;
        if data.0 >= tps.temp - 25.0 && data.0 <= tps.temp + 25.0 {
            index = i;
        }
    }
    wtr.flush()?;


    let output_file = path.clone() + "csv/" + &tps.name + "_max.csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Part", "Heat Capacity", "Thermal Insulance", "Emissivity"])?;
    
    for (i, data) in tps.data_max.iter().enumerate() {
        wtr.serialize((data.0, data.1.cp, 1.0 / data.1.R_th, data.1.e))?;
        if data.0 >= tps.temp - 25.0 && data.0 <= tps.temp + 25.0 {
            index = i;
        }
    }
    wtr.flush()?;

    // write layer into strucure Folder
    let directory = (&path).to_string() + "Debug_Info/" + &tps.name;
    fs::create_dir_all(&directory)?;
    for layer in tps.segments_min.clone() {
        output_layer(&layer,&directory)?;
    }

    let directory = path.clone() + "TPS/";
    let output_file = directory.to_owned() + &tps.name + ".cfg";
    fs::create_dir_all(&directory)?;
    let mut file = File::create(output_file)?;

    writeln!(file, "ROThermal_PRESET\n{{")?;
    writeln!(file, "    name = {}" , tps.name)?;
    writeln!(file, "    description = {}" , "")?;
    writeln!(file, "    type = Skin\n")?;

    writeln!(file, "    skinMaxTemp = {}" , tps.temp)?;
    writeln!(file, "    emissiveConstant = {}" , tps.data_min[index].1.e)?;
    writeln!(file, "    absorptiveConstant = {}\n" , tps.absorbation_const)?;

    writeln!(file, "    skinHeightMin = {:0.4}" , tps.tickness_min)?;
    writeln!(file, "    skinMassPerArea = {}" , tps.areal_density_min)?;
    writeln!(file, "    skinSpecificHeatCapacity = {}" , tps.data_min[index].1.cp)?;
    writeln!(file, "    thermalInsulance = {}\n" , tps.data_min[index].1.R_th)?;

    writeln!(file, "    skinHeightMax = {:0.4}" , tps.tickness_max)?;
    writeln!(file, "    skinMassPerAreaMax = {}" , tps.areal_density_max)?;
    writeln!(file, "    skinSpecificHeatCapacityMax = {}" , tps.data_max[index].1.cp)?;
    writeln!(file, "    thermalInsulanceMax = {}\n" , tps.data_max[index].1.R_th)?;

    writeln!(file, "    disableModAblator = {}" , tps.has_ablator)?;
    writeln!(file, "    costPerArea = {}" , tps.cost_per_area)?;
    writeln!(file, "}}")?;

    writeln!(file, "// Min: \n// Segment, Height")?;
    for segment in &tps.segments_min {
        writeln!(file, "// {}, {}",&segment.name, segment.tickness)?;    
    }
    writeln!(file, "\n// Max: \n// Segment, Height")?;
    for segment in &tps.segments_max {
        writeln!(file, "// {}, {}",&segment.name, segment.tickness)?;    
    }

    Ok(())
}

pub fn output_part(part: Part, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;
    fs::create_dir_all(path.to_owned() + "csv/")?;
    let mut index = 0;

    let output_file = path.clone() + "csv/" + &part.name + "_min.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while writing {}: {}", path.clone() + "csv/" + &part.name + "_min.csv", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity"])?;
    
    for (i, data) in part.data_min.iter().enumerate() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
        if data.0 >= part.temp - 25.0 && data.0 <= part.temp + 25.0 {
            index = i;
        }
    }
    wtr.flush()?;


    let output_file = path.to_owned()+ "csv/" + &part.name + "_max.csv";
    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while writing {}: {}", path.to_owned() + "csv/" + &part.name + "_max.csv", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity"])?;
    
    for data in part.data_max.iter() {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
    }
    wtr.flush()?;

    let directory = path.to_owned()+ "Part/";
    let output_file = directory.to_owned() + &part.name + ".cfg";
    fs::create_dir_all(&directory)?;
    let mut file = File::create(output_file)?;
    
    writeln!(file, "ROThermal_PRESET\n{{")?;
    writeln!(file, "    name = {}" , part.name)?;
    writeln!(file, "    description = {}" , part.description)?;
    writeln!(file, "    type = Skin\n")?;

    writeln!(file, "    skinMaxTemp = {}" , part.temp)?;
    writeln!(file, "    emissiveConstant = {}" , part.data_min[index].1.e)?;
    writeln!(file, "    absorptiveConstant = {}\n" , part.absorbation_const)?;

    writeln!(file, "    skinHeightMin = {:0.4}" , part.height_min)?;
    writeln!(file, "    skinMassPerArea = {}" , part.areal_density_min)?;
    writeln!(file, "    skinSpecificHeatCapacity = {}" , part.data_min[index].1.cp)?;
    writeln!(file, "    thermalInsulance = {}\n" , f32::powf(part.data_min[index].1.R_th, -1.0))?;

    writeln!(file, "    skinHeightMax = {:0.4}" , part.height_max)?;
    writeln!(file, "    skinMassPerAreaMax = {}" , part.areal_density_max)?;
    writeln!(file, "    skinSpecificHeatCapacityMax = {}" , part.data_max[index].1.cp)?;
    writeln!(file, "    thermalInsulanceMax = {}\n" , f32::powf(part.data_max[index].1.R_th, -1.0))?;

    writeln!(file, "    disableModAblator = {}" , part.has_ablator)?;
    writeln!(file, "    costPerArea = {}" , part.cost_per_area)?;
    writeln!(file, "}}")?;

    writeln!(file, "// Segment, Portion, Min Height, Max Height")?;
    for structure in &part.tps_list {
        writeln!(file, "// {}, {}, {}, {}",&structure.0.name, structure.1, &structure.0.tickness_min, &structure.0.tickness_max)?;    
    }

    Ok(())
}

pub fn output_data_triplet(name: &String, data: &Vec<DataTriplet>, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;

    let output_file = path.clone() + "/" + &name + ".csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity", "Temp Layer"])?;
    
    for data in data.iter() {
        wtr.serialize((data.temp_part, data.thermal_data.cp, data.thermal_data.R_th, data.thermal_data.e, data.temp_sub_part))?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn output_data_pair(name: &String, data: &Vec<DataPair>, path: String) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&path)?;

    let output_file = path.clone() + "/" + &name + ".csv";

    let mut wtr = match csv::Writer::from_path(&output_file){
        Ok(result) => {result},
        Err(err) =>  {println!("Error while reading Results.csv {}", err);
                            process::exit(1);}
    };

    wtr.write_record(&["Temp Part", "Heat Capacity", "1 / Thermal Insulance", "Emissivity"])?;
    
    for data in data {
        wtr.serialize((data.0, data.1.cp, data.1.R_th, data.1.e))?;
    }
    wtr.flush()?;
    Ok(())
}
