// Use wee_alloc as the global allocator to avoid dlmalloc issues
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Import console_error_panic_hook for better error messages
extern crate console_error_panic_hook;

use worker::*;

// Define modules
mod db;
mod scraper;
mod handlers;
mod utils;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    utils::log_request(&req);
    console_error_panic_hook::set_once();

    // Create the router for normal HTTP requests
    let router = Router::new();
    router
        .get("/", |_, _| Response::redirect(Url::parse("./graph").unwrap()))
        .get_async("/scrape", |req, ctx| {
            let env = ctx.env.clone();
            async move {
                handlers::scrape_handler(req, env).await
            }
        })
        .get_async("/history", |req, ctx| {
            let env = ctx.env.clone();
            async move {
                handlers::history_handler(req, env).await
            }
        })
        .get_async("/history/latest", |req, ctx| {
            let env = ctx.env.clone();
            async move {
                handlers::latest_handler(req, env).await
            }
        })
        .get_async("/websites", |req, ctx| {
            async move {
                handlers::websites_handler(req, ctx.env).await
            }
        })
        .get_async("/graph", |req, ctx| {
            let env = ctx.env.clone();
            async move {
                handlers::graph_handler(req, env).await
            }
        })
        .get_async("/time-averages", |req, ctx| {
            let env = ctx.env.clone();
            async move {
                let website_url = req.url()?.query_pairs()
                    .find(|(key, _)| key == "url")
                    .map(|(_, value)| value.to_string());
                
                match db::get_time_averages(&env, website_url.as_deref()).await {
                    Ok(data) => Response::from_json(&data),
                    Err(e) => Response::error(format!("Error fetching time averages: {}", e), 500)
                }
            }
        })
        .get_async("/time-averages-view", |req, ctx| {
            let env = ctx.env.clone();
            async move {
                handlers::time_averages_view_handler(req, env).await
            }
        })
        .run(req, env)
        .await
}

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    // Get the cron pattern from the event
    let cron = event.cron().to_string();
    
    // Delegate to the scheduled handler
    match handlers::scheduled::scheduled_handler(event, env, cron).await {
        Ok(_) => console_log!("Scheduled handler completed successfully"),
        Err(e) => console_error!("Error in scheduled handler: {}", e),
    }
} 