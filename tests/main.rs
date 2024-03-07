use tibber_api::subscription::Measurement;
use tibber_api::subscription::StreamExt;
use tibber_api::TibberClientError;

#[test_log::test(tokio::test)]
async fn test_demo_query_homes() {
    let client = tibber_api::TibberClient::new_demo();

    assert!(matches!(client.query_homes().await, Ok(..)));
}

async fn get_home() -> Result<Option<String>, TibberClientError> {
    let client = tibber_api::TibberClient::new_demo();
    client.query_homes().await.map(|mut r| r.pop())
}

#[test_log::test(tokio::test)]
async fn test_demo_subscribe_live() {
    let client = tibber_api::TibberClient::new_demo();

    let mut s = client
        .subscribe_live_measurement(get_home().await.unwrap().unwrap())
        .await
        .unwrap();

    for _ in 1..10 {
        assert!(matches!(s.next().await, Some(Ok(Measurement { .. }))));
    }
}

#[test_log::test(tokio::test)]
async fn test_demo_subscribe_test() {
    let client = tibber_api::TibberClient::new_demo();

    #[allow(deprecated)]
    let mut s = client
        .subscribe_test_measurement(get_home().await.unwrap().unwrap())
        .await
        .unwrap();

    for i in 1..10 {
        let Measurement {
            power,
            accumulated_consumption_last_hour,
            ..
        } = s.next().await.unwrap().unwrap();

        assert_eq!(i as f64, power);
        assert_eq!(0., accumulated_consumption_last_hour);
    }
}
