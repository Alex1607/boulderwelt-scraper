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
        .get("/", |_, _| Response::ok("Boulder Scraper Worker"))
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
        .run(req, env)
        .await
}

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    // Delegate to the scheduled handler
    let cron = "*/10 * * * *".to_string(); // Default to every 10 minutes
    match handlers::scheduled::scheduled_handler(event, env, cron).await {
        Ok(_) => console_log!("Scheduled handler completed successfully"),
        Err(e) => console_error!("Error in scheduled handler: {}", e),
    }
} 