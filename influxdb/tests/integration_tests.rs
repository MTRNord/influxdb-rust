extern crate influxdb;

#[path = "./utilities.rs"]
mod utilities;
use utilities::{
    assert_result_err, assert_result_ok, create_client, create_db, delete_db, run_test,
};

use influxdb::InfluxDbWriteable;
use influxdb::{Client, Error, Query, Timestamp};
use tokio;

/// INTEGRATION TEST
///
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it
#[tokio::test]
async fn test_ping_influx_db() {
    let client = create_client("notusedhere");
    let result = client.ping().await;
    assert_result_ok(&result);

    let (build, version) = result.unwrap();
    assert!(!build.is_empty(), "Build should not be empty");
    assert!(!version.is_empty(), "Build should not be empty");

    println!("build: {} version: {}", build, version);
}

/// INTEGRATION TEST
///
/// This test case tests connection error
#[tokio::test]
async fn test_connection_error() {
    let test_name = "test_connection_error";
    let client =
        Client::new("http://localhost:10086", test_name).with_auth("nopriv_user", "password");
    let read_query = Query::raw_read_query("SELECT * FROM weather");
    let read_result = client.query(&read_query).await;
    assert_result_err(&read_result);
    match read_result {
        Err(Error::ConnectionError { .. }) => {}
        _ => panic!(format!(
            "Should cause a ConnectionError: {}",
            read_result.unwrap_err()
        )),
    }
}

/// INTEGRATION TEST
///
/// This test case tests the Authentication
#[tokio::test]
async fn test_authed_write_and_read() {
    const TEST_NAME: &str = "test_authed_write_and_read";

    run_test(
        || async move {
            let client =
                Client::new("http://localhost:9086", TEST_NAME).with_auth("admin", "password");
            let query = format!("CREATE DATABASE {}", TEST_NAME);
            client
                .query(&Query::raw_read_query(query))
                .await
                .expect("could not setup db");

            let client =
                Client::new("http://localhost:9086", TEST_NAME).with_auth("admin", "password");
            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = client.query(&write_query).await;
            assert_result_ok(&write_result);

            let read_query = Query::raw_read_query("SELECT * FROM weather");
            let read_result = client.query(&read_query).await;
            assert_result_ok(&read_result);
            assert!(
                !read_result.unwrap().contains("error"),
                "Data contained a database error"
            );
        },
        || async move {
            let client =
                Client::new("http://localhost:9086", TEST_NAME).with_auth("admin", "password");
            let query = format!("DROP DATABASE {}", TEST_NAME);

            client
                .query(&Query::raw_read_query(query))
                .await
                .expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This test case tests the Authentication
#[tokio::test]
async fn test_wrong_authed_write_and_read() {
    const TEST_NAME: &str = "test_wrong_authed_write_and_read";

    run_test(
        || async move {
            let client =
                Client::new("http://localhost:9086", TEST_NAME).with_auth("admin", "password");
            let query = format!("CREATE DATABASE {}", TEST_NAME);
            client
                .query(&Query::raw_read_query(query))
                .await
                .expect("could not setup db");

            let client =
                Client::new("http://localhost:9086", TEST_NAME).with_auth("wrong_user", "password");
            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = client.query(&write_query).await;
            assert_result_err(&write_result);
            match write_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(format!(
                    "Should be an AuthorizationError: {}",
                    write_result.unwrap_err()
                )),
            }

            let read_query = Query::raw_read_query("SELECT * FROM weather");
            let read_result = client.query(&read_query).await;
            assert_result_err(&read_result);
            match read_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(format!(
                    "Should be an AuthorizationError: {}",
                    read_result.unwrap_err()
                )),
            }

            let client = Client::new("http://localhost:9086", TEST_NAME)
                .with_auth("nopriv_user", "password");
            let read_query = Query::raw_read_query("SELECT * FROM weather");
            let read_result = client.query(&read_query).await;
            assert_result_err(&read_result);
            match read_result {
                Err(Error::AuthenticationError) => {}
                _ => panic!(format!(
                    "Should be an AuthenticationError: {}",
                    read_result.unwrap_err()
                )),
            }
        },
        || async move {
            let client =
                Client::new("http://localhost:9086", TEST_NAME).with_auth("admin", "password");
            let query = format!("DROP DATABASE {}", TEST_NAME);
            client
                .query(&Query::raw_read_query(query))
                .await
                .expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This test case tests the Authentication
#[tokio::test]
async fn test_non_authed_write_and_read() {
    const TEST_NAME: &str = "test_non_authed_write_and_read";

    run_test(
        || async move {
            let client =
                Client::new("http://localhost:9086", TEST_NAME).with_auth("admin", "password");
            let query = format!("CREATE DATABASE {}", TEST_NAME);
            client
                .query(&Query::raw_read_query(query))
                .await
                .expect("could not setup db");
            let non_authed_client = Client::new("http://localhost:9086", TEST_NAME);
            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = non_authed_client.query(&write_query).await;
            assert_result_err(&write_result);
            match write_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(format!(
                    "Should be an AuthorizationError: {}",
                    write_result.unwrap_err()
                )),
            }

            let read_query = Query::raw_read_query("SELECT * FROM weather");
            let read_result = non_authed_client.query(&read_query).await;
            assert_result_err(&read_result);
            match read_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(format!(
                    "Should be an AuthorizationError: {}",
                    read_result.unwrap_err()
                )),
            }
        },
        || async move {
            let client =
                Client::new("http://localhost:9086", TEST_NAME).with_auth("admin", "password");
            let query = format!("DROP DATABASE {}", TEST_NAME);
            client
                .query(&Query::raw_read_query(query))
                .await
                .expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This integration tests that writing data and retrieving the data again is working
#[tokio::test]
async fn test_write_and_read_field() {
    const TEST_NAME: &str = "test_write_field";

    run_test(
        || async move {
            create_db(TEST_NAME).await.expect("could not setup db");
            let client = create_client(TEST_NAME);
            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = client.query(&write_query).await;
            assert_result_ok(&write_result);

            let read_query = Query::raw_read_query("SELECT * FROM weather");
            let read_result = client.query(&read_query).await;
            assert_result_ok(&read_result);
            assert!(
                !read_result.unwrap().contains("error"),
                "Data contained a database error"
            );
        },
        || async move {
            delete_db(TEST_NAME).await.expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This integration tests that writing data and retrieving the data again is working
#[tokio::test]
#[cfg(feature = "use-serde")]
async fn test_write_and_read_option() {
    use serde::Deserialize;

    const TEST_NAME: &str = "test_write_and_read_option";

    run_test(
        || {
            async move {
                create_db(TEST_NAME).await.expect("could not setup db");

                let client = create_client(TEST_NAME);
                // Todo: Convert this to derive based insert for easier comparison of structs
                let write_query = Timestamp::Hours(11)
                    .into_query("weather")
                    .add_field("temperature", 82)
                    .add_field("wind_strength", <Option<u64>>::None);
                let write_result = client.query(&write_query).await;
                assert_result_ok(&write_result);

                #[derive(Deserialize, Debug, PartialEq)]
                struct Weather {
                    time: String,
                    temperature: i32,
                    wind_strength: Option<u64>,
                }

                let query =
                    Query::raw_read_query("SELECT time, temperature, wind_strength FROM weather");
                let result = client
                    .json_query(query)
                    .await
                    .and_then(|mut db_result| db_result.deserialize_next::<Weather>());
                assert_result_ok(&result);

                assert_eq!(
                    result.unwrap().series[0].values[0],
                    Weather {
                        time: "1970-01-01T11:00:00Z".to_string(),
                        temperature: 82,
                        wind_strength: None,
                    }
                );
            }
        },
        || async move {
            delete_db("test_write_and_read_option")
                .await
                .expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response and whether that JSON
/// is equal to the data which was written to the database
#[tokio::test]
#[cfg(feature = "use-serde")]
async fn test_json_query() {
    use serde::Deserialize;

    const TEST_NAME: &str = "test_json_query";

    run_test(
        || async move {
            create_db(TEST_NAME).await.expect("could not setup db");

            let client = create_client(TEST_NAME);

            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = client.query(&write_query).await;
            assert_result_ok(&write_result);

            #[derive(Deserialize, Debug, PartialEq)]
            struct Weather {
                time: String,
                temperature: i32,
            }

            let query = Query::raw_read_query("SELECT * FROM weather");
            let result = client
                .json_query(query)
                .await
                .and_then(|mut db_result| db_result.deserialize_next::<Weather>());
            assert_result_ok(&result);

            assert_eq!(
                result.unwrap().series[0].values[0],
                Weather {
                    time: "1970-01-01T11:00:00Z".to_string(),
                    temperature: 82
                }
            );
        },
        || async move {
            delete_db(TEST_NAME).await.expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response and wether that JSON
/// is equal to the data which was written to the database
#[tokio::test]
#[cfg(feature = "use-serde")]
async fn test_json_query_vec() {
    use serde::Deserialize;

    const TEST_NAME: &str = "test_json_query_vec";

    run_test(
        || async move {
            create_db(TEST_NAME).await.expect("could not setup db");

            let client = create_client(TEST_NAME);
            let write_query1 = Timestamp::Hours(11)
                .into_query("temperature_vec")
                .add_field("temperature", 16);
            let write_query2 = Timestamp::Hours(12)
                .into_query("temperature_vec")
                .add_field("temperature", 17);
            let write_query3 = Timestamp::Hours(13)
                .into_query("temperature_vec")
                .add_field("temperature", 18);

            let _write_result = client.query(&write_query1).await;
            let _write_result2 = client.query(&write_query2).await;
            let _write_result2 = client.query(&write_query3).await;

            #[derive(Deserialize, Debug, PartialEq)]
            struct Weather {
                time: String,
                temperature: i32,
            }

            let query = Query::raw_read_query("SELECT * FROM temperature_vec");
            let result = client
                .json_query(query)
                .await
                .and_then(|mut db_result| db_result.deserialize_next::<Weather>());
            assert_result_ok(&result);
            assert_eq!(result.unwrap().series[0].values.len(), 3);
        },
        || async move {
            delete_db(TEST_NAME).await.expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This integration test tests whether using the wrong query method fails building the query
#[tokio::test]
#[cfg(feature = "use-serde")]
async fn test_serde_multi_query() {
    use serde::Deserialize;

    const TEST_NAME: &str = "test_serde_multi_query";

    run_test(
        || async move {
            create_db(TEST_NAME).await.expect("could not setup db");

            #[derive(Deserialize, Debug, PartialEq)]
            struct Temperature {
                time: String,
                temperature: i32,
            }

            #[derive(Deserialize, Debug, PartialEq)]
            struct Humidity {
                time: String,
                humidity: i32,
            }

            let client = create_client(TEST_NAME);
            let write_query = Timestamp::Hours(11)
                .into_query("temperature")
                .add_field("temperature", 16);
            let write_query2 = Timestamp::Hours(11)
                .into_query("humidity")
                .add_field("humidity", 69);

            let write_result = client.query(&write_query).await;
            let write_result2 = client.query(&write_query2).await;
            assert_result_ok(&write_result);
            assert_result_ok(&write_result2);

            let result = client
                .json_query(
                    Query::raw_read_query("SELECT * FROM temperature")
                        .add_query("SELECT * FROM humidity"),
                )
                .await
                .and_then(|mut db_result| {
                    let temp = db_result.deserialize_next::<Temperature>()?;
                    let humidity = db_result.deserialize_next::<Humidity>()?;

                    Ok((temp, humidity))
                });
            assert_result_ok(&result);

            let (temp, humidity) = result.unwrap();
            assert_eq!(
                temp.series[0].values[0],
                Temperature {
                    time: "1970-01-01T11:00:00Z".to_string(),
                    temperature: 16
                },
            );
            assert_eq!(
                humidity.series[0].values[0],
                Humidity {
                    time: "1970-01-01T11:00:00Z".to_string(),
                    humidity: 69
                }
            );
        },
        || async move {
            delete_db(TEST_NAME).await.expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This integration test tests whether using the wrong query method fails building the query
#[tokio::test]
#[cfg(feature = "use-serde")]
async fn test_wrong_query_errors() {
    let client = create_client("test_name");
    let result = client
        .json_query(Query::raw_read_query("CREATE DATABASE this_should_fail"))
        .await;
    assert!(
        result.is_err(),
        "Should only build SELECT and SHOW queries."
    );
}

/// INTEGRATION TEST
///
/// Test if async-std is available when asynchronous
#[test]
fn test_async_std_runtime() {
    async_std::task::block_on(async {
        let test_name = "mydb";
        let client = Client::new("http://localhost:8086", test_name);
        let read_query = Query::raw_read_query("SELECT * FROM \"weather\"");
        let read_result = client.query(&read_query).await;
        assert!(read_result.is_ok());
    });
}
