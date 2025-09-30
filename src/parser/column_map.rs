use csv::StringRecord;

pub struct ColumnMap {
    pub name: usize,
    pub code: usize,
    pub country: usize,
    pub lat: usize,
    pub lon: usize,
    pub elev: usize,
    pub style: usize,
    pub rwdir: Option<usize>,
    pub rwlen: Option<usize>,
    pub rwwidth: Option<usize>,
    pub freq: Option<usize>,
    pub desc: Option<usize>,
    pub userdata: Option<usize>,
    pub pics: Option<usize>,
}

pub fn build_column_map(headers: &StringRecord) -> Result<ColumnMap, String> {
    let mut name = None;
    let mut code = None;
    let mut country = None;
    let mut lat = None;
    let mut lon = None;
    let mut elev = None;
    let mut style = None;
    let mut rwdir = None;
    let mut rwlen = None;
    let mut rwwidth = None;
    let mut freq = None;
    let mut desc = None;
    let mut userdata = None;
    let mut pics = None;

    for (idx, header) in headers.iter().enumerate() {
        match header.to_lowercase().as_str() {
            "name" => name = Some(idx),
            "code" => code = Some(idx),
            "country" => country = Some(idx),
            "lat" => lat = Some(idx),
            "lon" => lon = Some(idx),
            "elev" => elev = Some(idx),
            "style" => style = Some(idx),
            "rwdir" => rwdir = Some(idx),
            "rwlen" => rwlen = Some(idx),
            "rwwidth" => rwwidth = Some(idx),
            "freq" => freq = Some(idx),
            "desc" => desc = Some(idx),
            "userdata" => userdata = Some(idx),
            "pics" => pics = Some(idx),
            _ => {}
        }
    }

    Ok(ColumnMap {
        name: name.ok_or("Missing required column: name")?,
        code: code.ok_or("Missing required column: code")?,
        country: country.ok_or("Missing required column: country")?,
        lat: lat.ok_or("Missing required column: lat")?,
        lon: lon.ok_or("Missing required column: lon")?,
        elev: elev.ok_or("Missing required column: elev")?,
        style: style.ok_or("Missing required column: style")?,
        rwdir,
        rwlen,
        rwwidth,
        freq,
        desc,
        userdata,
        pics,
    })
}
