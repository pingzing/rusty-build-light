
//  public class CoinbaseResponse
//     {
//         public CoinbaseResponseData Data { get; set; }
//     }

//     public class CoinbaseResponseData
//     {
//         public string Amount { get; set; }
//         public string Currency { get; set; }
//     }

#[derive(Deserialize)]
pub struct CoinbaseResponse {
    pub data: CoinbaseResponseData
}

#[derive(Deserialize)]
pub struct CoinbaseResponseData{
    pub amount: String,
    pub currency: String
}

