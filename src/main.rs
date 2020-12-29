// Receive NATS indoor environment messages, and forward to InfluxDB.
use influxdb::InfluxDbWriteable;
use chrono::{DateTime, TimeZone, Utc};

// time, location, sensorModel are mandatory
#[derive(InfluxDbWriteable)]
    struct IndoorEnvReading {
        time: DateTime<Utc>,
        temp: Option<f64>,
        humidity: Option<f64>,
        pressure: Option<f64>,
        #[tag] location: String,
        #[tag] sensorModel: String,
    }

#[async_std::main]
async fn main() {
    println!("Connecting to NATS");
    let ncres = nats::connect("nats.wellorder.net");
    let nc = match ncres {
        Ok(conn) => conn,
        Err(_e) => {
            println!("Could not connect, bailing");
            std::process::exit(1);
        }
    };
    println!("Subscribing to iot.indoorenv topic");
    let subres = nc.subscribe("iot.indoorenv");
    let sub = match subres {
        Ok(s) => s,
        Err(_e) => {
            println!("Could not get subscription, bailing");
            std::process::exit(1);
        }
    };
    // Connect to influxdb
    println!("Connecting to InfluxDB");
    let client = influxdb::Client::new("http://ektar.wellorder.net:8086", "iot");
    for msg in sub.messages() {
        //println!("This message subject is: {}", msg.subject);
        let utf8res = std::str::from_utf8(&msg.data);
        let msgstr = match utf8res {
            Ok(s) => s,
            Err(_e) => { std::process::exit(1) }
        };
        println!("Message is: {}", msgstr);
        // Build a JSON deserializer for the message
        let event : cloudevents::event::Event = serde_json::from_str(msgstr).unwrap();
        println!("{}", event);
        let payload = match event.data().unwrap() {
            cloudevents::Data::Json(v) => v,
            _ => { 
                println!("Did not match JSON payload");
                std::process::exit(1);
            }
        }; 
        println!("{}", payload);
        // extract fields from payload
        let mainobj = match payload {
            serde_json::value::Value::Object(m) => m,
            _ => {
                println!("Expected a top-level object");
                std::process::exit(1);
            }
        };
        // extract temp from mainobj
        let temp = mainobj.get("temp").and_then(serde_json::Value::as_f64);
        // humiditiy;
        let humidity = mainobj.get("humidity").and_then(serde_json::Value::as_f64);
        // pressure
        let pressure = mainobj.get("pressure").and_then(serde_json::Value::as_f64);
        // location
        let location = mainobj.get("loc").and_then(serde_json::Value::as_str).map(|x| x.to_string()).unwrap();
        // sensor model
        let sensorModel = mainobj.get("sensorModel").and_then(serde_json::Value::as_str).map(|x| x.to_string()).unwrap();
        // parse the data payload
        let dtflt = mainobj.get("dt").unwrap().as_f64().unwrap();
        // Get second component
        let dtsec = dtflt as i64;
        // Get nanoseconds
        let dtnano = ((dtflt - (dtsec as f64)) * 1e9) as u32;
        let dt = Utc.timestamp(dtsec, dtnano);
        let wr = IndoorEnvReading {
            time: dt,
            temp: temp,
            humidity: humidity,
            pressure: pressure,
            location: location,
            sensorModel: sensorModel
        }; 
        let write_result = client
            .query(&wr.into_query("indoorenv")).await;
        assert!(write_result.is_ok(), "Write result to influxdb was not okay");
        //let vr: Result<serde_json::Value, serde_json::error::Error> = serde_json::from_str(event.data().unwrap());
//        event.deserialize(msgstr)
//        let parsed_event = serde::from_str(msgstr).unwrap();

        // Need to run iter_attributes over the parsed Event
    }

}
