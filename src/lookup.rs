//! Structs, functions, etc for working with Twilio's Lookup v2 API endpoint

use cli_table::{Cell, Style, Table, print_stdout};
use itertools::Itertools;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

/// This models the phone number information that is returned from requests to the API
///
/// <div class="warning">
/// Currently, the struct only models some of the properties.
/// The remaining properties will be added in future versions of the crate.
/// Specifically, it only models the core properties plus properties of the Sim Swap and
/// Line Type Intelligence add on packages.
/// </div>
///
/// You can find complete documentation about all of the available properties
/// [in the documentation](https://www.twilio.com/docs/lookup/v2-api#response-properties).
#[derive(Debug, Default, Deserialize)]
pub struct PhoneNumber {
    pub country_code: Option<String>,
    pub phone_number: Option<String>,
    pub national_format: Option<String>,
    pub valid: bool,
    pub validation_errors: Option<Vec<String>>,
    pub sim_swap: Option<SimSwap>,
    pub line_type_intelligence: Option<LineTypeIntelligence>,
    pub sms_pumping_risk: Option<SmsPumpingRiskScore>,
}

impl PhoneNumber {
    /// Returns whether the phone number is valid or not
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Returns any validation errors requesting information about the phone number
    pub fn get_validation_errors(&self) -> Option<Vec<String>> {
        self.validation_errors.clone()
    }

    /// Returns whether line type intelligence information is available for the phone number
    pub fn has_line_type_intelligence(&self) -> bool {
        self.line_type_intelligence.is_some()
    }

    /// Retrieves the line type intelligence information for a phone number
    pub fn get_line_type_intelligence(&self) -> LineTypeIntelligence {
        let line_type_intelligence = match &self.line_type_intelligence {
            Some(line_type_intelligence) => line_type_intelligence,
            _ => &LineTypeIntelligence::default(),
        };

        line_type_intelligence.clone()
    }

    /// Retrieves the SMS pumping risk score information for a phone number
    ///
    /// # Examples
    ///
    /// ## Basic example
    ///
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use rustlio::lookup;
    /// use std::collections::HashMap;
    /// use std::env;
    ///
    /// # tokio_test::block_on(async {
    /// let phone_number = "+61123456789";
    ///
    /// // Setting a DataPackage to true indicates that it is to be in the lookup request.
    /// // You can add a DataPackage and set it to false, or just not include it to not
    /// // include it in the lookup request.
    /// let data_packages = HashMap::from([
    ///     (lookup::DataPackage::SmsPumpingRiskScore, true),
    /// ]);
    ///
    /// // Perform a basic lookup of a phone number
    /// // This will only return the basic details of the phone number, without any extra,
    /// // Data Package, information.
    /// let data = lookup::lookup_phone_data(
    ///     phone_number,
    ///     &data_packages,
    ///     &Client::new(),
    ///     &env::var("TWILIO_ACCOUNT_SID").unwrap(),
    ///     &env::var("TWILIO_AUTH_TOKEN").unwrap()
    /// )
    /// .await
    /// .expect(format!("Unable to retrieve data for {phone_number}").as_str());
    ///
    /// // If the phone number was valid, print out the carrier risk category, if available.
    /// if data.is_valid() {
    ///     let sms_pumping_risk = data.get_sms_pumping_risk_score();
    ///     if let Some(carrier_risk_category) = sms_pumping_risk.carrier_risk_category {
    ///         println!("The phone number's carrier risk category is: {carrier_risk_category}");
    ///     }
    /// } else {
    ///     println!("{phone_number} is NOT valid")
    /// }
    /// # })
    /// ```
    pub fn get_sms_pumping_risk_score(&self) -> SmsPumpingRiskScore {
        let sms_pumping_risk = match &self.sms_pumping_risk {
            Some(sms_pumping_risk) => sms_pumping_risk,
            _ => &SmsPumpingRiskScore::default(),
        };

        sms_pumping_risk.clone()
    }

    /// Returns the phone number's line type
    ///
    /// This can be one of: landline, mobile, fixedVoip, nonFixedVoip, personal tollFree,
    /// premium, sharedCost, uan, voicemail, pager, and unknown.
    pub fn get_line_type(&self) -> String {
        let line_type_intelligence = match &self.line_type_intelligence {
            Some(line_type_intelligence) => line_type_intelligence,
            _ => &LineTypeIntelligence::default(),
        };

        line_type_intelligence
            .line_type
            .clone()
            .unwrap_or("Unknown line type".to_string())
    }

    /// Returns whether a SIM has been swapped recently
    pub fn has_been_swapped(&self) -> bool {
        match &self.sim_swap {
            Some(sim_swap) => match &sim_swap.last_sim_swap {
                Some(last_sim_swap) => {
                    last_sim_swap.last_sim_swap_date.is_some()
                        && last_sim_swap.swapped_period.is_some()
                        && last_sim_swap.swapped_in_period
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Returns the phone number's SIM swap data if available
    pub fn get_sim_swap_data(&self) -> LastSimSwap {
        let sim_swap = match &self.sim_swap {
            Some(sim_swap) => sim_swap,
            _ => &SimSwap::default(),
        };

        let last_sim_swap = match &sim_swap.last_sim_swap {
            Some(last_sim_swap_data) => last_sim_swap_data,
            _ => &LastSimSwap::default(),
        };

        last_sim_swap.clone()
    }
}

/// LineTypeIntelligence models a phone number's line type intelligence properties
///
/// For more information, [check out the documentation](https://www.twilio.com/docs/lookup/v2-api/line-type-intelligence).
#[derive(Clone, Deserialize, Default, Debug)]
pub struct LineTypeIntelligence {
    #[serde(rename = "type")]
    pub line_type: Option<String>,
    pub mobile_country_code: Option<String>,
    pub mobile_network_code: Option<String>,
    pub carrier_name: Option<String>,
    pub error_code: Option<String>,
}

/// SmsPumpingRiskScore models a phone number's SMS pumping risk score properties
///
/// For more information, [check out the documentation](https://www.twilio.com/docs/lookup/v2-api/sms-pumping-risk).
#[derive(Clone, Deserialize, Default, Debug)]
pub struct SmsPumpingRiskScore {
    pub carrier_risk_category: Option<String>,
    pub number_blocked: Option<bool>,
    pub number_blocked_date: Option<String>,
    pub number_blocked_last_3_months: Option<bool>,
    pub sms_pumping_risk_score: u32,
    pub error_code: Option<String>,
}

/// SimSwap models a phone number's sim swap properties
///
/// For more information, [check out the documentation](https://www.twilio.com/docs/lookup/v2-api/sim-swap).
#[derive(Default, Deserialize, Debug)]
pub struct SimSwap {
    pub last_sim_swap: Option<LastSimSwap>,
}

/// LastSimSwap models the last_sim_swap properties of a phone number's sim_swap property
///
/// For more information, [check out the documentation](https://www.twilio.com/docs/lookup/v2-api/sim-swap#lastsimswap-property-values).
#[derive(Clone, Default, Deserialize, Debug)]
pub struct LastSimSwap {
    pub last_sim_swap_date: Option<String>,
    pub swapped_period: Option<String>,
    pub swapped_in_period: bool,
}

impl LastSimSwap {
    // This returns whether a SIM was swapped recently
    pub fn was_swapped_recently(&self) -> bool {
        self.swapped_in_period
    }
}

/// DataPackage models all of the available add on packages which the Lookup v2 API supports
///
/// For more information, [check out the documentation](https://www.twilio.com/docs/lookup/v2-api).
#[derive(PartialEq, Eq, Hash)]
pub enum DataPackage {
    CallForwarding,
    CallerName,
    IdentityMatch,
    LineStatus,
    LineTypeIntelligence,
    PhoneNumberQualityScore,
    PreFill,
    ReassignedNumber,
    SimSwap,
    SmsPumpingRiskScore,
    Validation,
}

impl DataPackage {
    // This maps the enum's properties to the respective values which the Lookup v2 API accepts
    pub fn as_str(&self) -> &'static str {
        match self {
            DataPackage::CallForwarding => "call_forwarding",
            DataPackage::CallerName => "caller_name",
            DataPackage::IdentityMatch => "identity_match",
            DataPackage::LineStatus => "line_status",
            DataPackage::LineTypeIntelligence => "line_type_intelligence",
            DataPackage::PhoneNumberQualityScore => "phone_number_quality_score",
            DataPackage::PreFill => "pre_fill",
            DataPackage::ReassignedNumber => "reassigned_number",
            DataPackage::SimSwap => "sim_swap",
            DataPackage::SmsPumpingRiskScore => "sms_pumping_risk",
            DataPackage::Validation => "validation",
        }
    }
}

/// Builds the URL required to make a request to Twilio's Lookup v2 API.
///
/// # Examples
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use rustlio::lookup::DataPackage;
/// use rustlio::lookup::get_lookup_request_url;
///
/// let mut data_packages: HashMap<DataPackage, bool> = HashMap::new();
/// data_packages.insert(DataPackage::LineTypeIntelligence, true);
/// data_packages.insert(DataPackage::SimSwap, true);
/// data_packages.insert(DataPackage::SmsPumpingRiskScore, true);
/// let phone_number = "+61123456789";
///
/// let url = get_lookup_request_url(
///     format!("https://lookups.twilio.com/v2/PhoneNumbers/{phone_number}"),
///     &data_packages
/// );
/// ```
pub fn get_lookup_request_url(
    lookup_uri_base: String,
    data_packages: &HashMap<DataPackage, bool>,
) -> Url {
    let mut issue_list_url =
        Url::parse(&lookup_uri_base).expect("Unable to parse the provided URL");

    if data_packages.is_empty() {
        return issue_list_url;
    }

    let desired_data_packages = data_packages.iter().filter_map(|(k, v)| match v {
        true => Some(k),
        false => None,
    });

    let field_names: String = desired_data_packages
        .map(|field| field.as_str())
        .sorted()
        .collect::<Vec<&str>>()
        .join(",");

    if !field_names.is_empty() {
        issue_list_url.set_query(Some(format!("Fields={field_names}").as_str()));
    }

    issue_list_url
}

/// Looks up details of a phone number using the Twilio Lookup v2 API
///
/// To do this, the function takes the phone number to perform the lookup for, a HashMap of
/// DataPackage to determine the add on packages to include in the request, and a set of Twilio
/// credentials (the Account SID and Auth Token for an account).
///
/// It returns a [`PhoneNumber`] if the phone number details were returned from the API request.
/// Otherwise, it returns an [Err](https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err).
///
/// # Examples
///
/// ## Basic usage with no Data Package information:
///
/// ```rust,no_run
/// use reqwest::Client;
/// use rustlio::lookup;
/// use std::collections::HashMap;
/// use std::env;
///
/// # tokio_test::block_on(async {
/// let phone_number = "+61123456789";
///
/// // Perform a basic lookup of a phone number
/// // This will only return the basic details of the phone number, without any extra,
/// // Data Package, information.
/// let data = lookup::lookup_phone_data(
///     phone_number,
///     &HashMap::new(),
///     &Client::new(),
///     &env::var("TWILIO_ACCOUNT_SID").unwrap(),
///     &env::var("TWILIO_AUTH_TOKEN").unwrap()
/// )
/// .await
/// .expect(format!("Unable to retrieve data for {phone_number}").as_str());
///
/// // Confirm whether the phone number was retrieved and valid or not
/// if data.is_valid() {
///     println!("{phone_number} is valid");
/// } else {
///     println!("{phone_number} is NOT valid")
/// }
/// # })
/// ```
///
/// ## Query phone number details including Line Type Intelligence details
///
/// ```rust,no_run
/// use reqwest::Client;
/// use rustlio::lookup;
/// use std::collections::HashMap;
/// use std::env;
///
/// # tokio_test::block_on(async {
/// let phone_number = "+61123456789";
///
/// // Setting a DataPackage to true indicates that it is to be in the lookup request.
/// // You can add a DataPackage and set it to false, or just not include it to not
/// // include it in the lookup request.
/// let data_packages = HashMap::from([
///     (lookup::DataPackage::LineTypeIntelligence, true),
/// ]);
///
/// // Perform a basic lookup of a phone number
/// // This will only return the basic details of the phone number, without any extra,
/// // Data Package, information.
/// let data = lookup::lookup_phone_data(
///     phone_number,
///     &data_packages,
///     &Client::new(),
///     &env::var("TWILIO_ACCOUNT_SID").unwrap(),
///     &env::var("TWILIO_AUTH_TOKEN").unwrap()
/// )
/// .await
/// .expect(format!("Unable to retrieve data for {phone_number}").as_str());
///
/// // If the phone number was valid, print out the carrier's name, if available.
/// if data.is_valid() {
///     let line_type_intelligence = data.get_line_type_intelligence();
///     if let Some(carrier_name) = line_type_intelligence.carrier_name {
///         println!("The phone number's carrier is: {carrier_name}");
///     }
/// } else {
///     println!("{phone_number} is NOT valid")
/// }
/// # })
/// ```
pub async fn lookup_phone_data(
    phone_number: &str,
    data_packages: &HashMap<DataPackage, bool>,
    client: &Client,
    user: &String,
    password: &String,
) -> Result<PhoneNumber, reqwest::Error> {
    let request_url: String = get_lookup_request_url(
        format!("https://lookups.twilio.com/v2/PhoneNumbers/{phone_number}"),
        data_packages,
    )
    .to_string();

    let response = client
        .get(request_url)
        .basic_auth(user, Some(password))
        .send()
        .await?;

    let result = response.error_for_status();
    match result {
        Ok(data) => {
            let record: PhoneNumber = data.json().await?;
            Ok(record)
        }
        Err(e) => Err(e),
    }
}

/// Retrieve a phone number with line type intelligence information
///
/// It's a small wrapper around the [`lookup_phone_data`] function to save time and effort calling
/// the function and passing in a HashMap with the LineTypeIntelligence DataPackage set.
/// See [`lookup_phone_data`] for more information.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use rustlio::lookup::PhoneNumber;
/// use rustlio::lookup::lookup_phone_data_with_line_type;
/// use std::env;
///
/// # tokio_test::block_on(async {
/// // Attempt to retrieve a phone number with line type intelligence information.
/// let data = lookup_phone_data_with_line_type(
///     "+61123456789",
///     &reqwest::Client::new(),
///     &env::var("TWILIO_ACCOUNT_SID").unwrap(),
///     &env::var("TWILIO_AUTH_TOKEN").unwrap()
/// ).await;
///
/// // Check if a valid phone number was returned, instantiating a default one, if not.
/// let phone_number = match data {
///     Ok(number) => number,
///     Err(_e) => PhoneNumber::default(),
/// };
///
/// // Check the phone number's line type.
/// println!("{}", phone_number.get_line_type());
/// # })
/// ```
pub async fn lookup_phone_data_with_line_type(
    phone_number: &str,
    client: &Client,
    user: &String,
    password: &String,
) -> Result<PhoneNumber, reqwest::Error> {
    lookup_phone_data(
        phone_number,
        &HashMap::from([(DataPackage::LineTypeIntelligence, true)]),
        client,
        user,
        password,
    )
    .await
}

/// A convenience method for displaying the validation errors for an invalid phone number
pub fn show_invalid_phone_number(phone_number: String, record: &PhoneNumber) {
    println!("{phone_number} is not a valid phone number. See the errors below to know why.");
    let errors = record.get_validation_errors().unwrap_or_default();
    for error in errors {
        println!("Error: {}", error);
    }
}

/// Takes a twilio::PhoneNumber and displays its details in tabular format
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,no_run
/// use rustlio::lookup;
///
/// // The below example is a simplistic example of instantiating a PhoneNumber.
/// // Another way is to use the lookup_phone_data function and retrieve live data.
/// let record: lookup::PhoneNumber = lookup::PhoneNumber {
///     country_code: Some("+61".to_string()),
///     phone_number: Some("+61123456789".to_string()),
///     national_format: Some("0123456789".to_string()),
///     valid: true,
///     validation_errors: None,
///     sim_swap: None,
///     line_type_intelligence: None,
///     sms_pumping_risk: None,
/// };
/// lookup::print_phone_number_data(&record);
/// ```
pub fn print_phone_number_data(record: &PhoneNumber) {
    let country_code: &String = record
        .country_code
        .as_ref()
        .expect("Was expecting a valid country code");
    let phone_number: &String = record
        .phone_number
        .as_ref()
        .expect("Was expecting a valid phone number");
    let national_format: &String = record
        .national_format
        .as_ref()
        .expect("Was expecting a valid national format for the phone number");

    let mut headers = vec![
        "Country Code".cell().bold(true),
        "Phone Number (E.164 format)".cell().bold(true),
        "Phone Number (national format)".cell().bold(true),
    ];

    let mut row = vec![
        country_code.cell(),
        phone_number.cell(),
        national_format.cell(),
    ];

    if record.has_been_swapped() {
        let last_sim_swap_data: LastSimSwap = record.get_sim_swap_data();
        headers.push("SIM Swap Data".cell().bold(true));
        let swap_date: String = last_sim_swap_data
            .last_sim_swap_date
            .unwrap_or("Swap date not available.".to_string());
        row.push(format!("SIM was swapped on: {}", swap_date).cell());
    }

    if record.has_line_type_intelligence() {
        let line_type_intelligence: LineTypeIntelligence = record.get_line_type_intelligence();
        headers.push("Line Type".cell().bold(true));
        row.push(
            line_type_intelligence
                .line_type
                .unwrap_or("Line type not available".to_string())
                .cell(),
        );
    }

    let table = vec![row].table().title(headers).bold(true);

    assert!(print_stdout(table).is_ok());
}
