use actix_web::{web, HttpResponse, Responder, HttpRequest , HttpMessage};
use clickhouse::Client;

use crate::models::auth::Claims;

use crate::models::report::ReportRow;
use crate::models::report::ReportPath;



pub async fn fetch_report_by_type(
    client: web::Data<Client>,
    path: web::Path<ReportPath>,
     req: HttpRequest,
) -> impl Responder {
    let report_type = &path.report_type;

     let extensions = req.extensions();
    let claims = extensions.get::<Claims>();
    if let Some(claims) = claims {
        if claims.role != "admin" {
            return HttpResponse::Forbidden().body("Only admin can view reports");
        }
    } else {
        return HttpResponse::Unauthorized().body("Unauthorized access");
    }

   
    let valid_types = ["daily", "weekly", "monthly", "yearly"];
    if !valid_types.contains(&report_type.as_str()) {
        return HttpResponse::BadRequest().body("Invalid report type");
    }

    let query = format!(
        "SELECT 
    toString(period) AS period,
    product_name,
    sumMerge(total_units_sold) AS total_units_sold,
    sumMerge(total_sales_amount) AS total_sales_amount
FROM product_sales_summary
WHERE report_type = '{}' 
GROUP BY period, product_name
ORDER BY period DESC ",
        report_type
    );

    match client.query(&query).fetch_all::<ReportRow>().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            eprintln!("Report fetch error for {}: {:?}", report_type, e);
            HttpResponse::InternalServerError().body("Failed to fetch report")
        }
    }
}
