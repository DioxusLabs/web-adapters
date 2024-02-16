
        #[cfg(all(feature = "warp", not(feature = "axum"), not(feature = "salvo")))]
        {
            use warp::Filter;
            // First register the server functions
            let router = register_server_fns(server_fn_route);
            #[cfg(not(any(feature = "desktop", feature = "mobile")))]
            let router = {
                // Serve the dist folder and the index.html file
                let serve_dir = warp::fs::dir(cfg.assets_path);
                let build_virtual_dom = Arc::new(build_virtual_dom);

                router
                    .or(connect_hot_reload())
                    // Then the index route
                    .or(warp::path::end().and(render_ssr(cfg.clone(), {
                        let build_virtual_dom = build_virtual_dom.clone();
                        move || build_virtual_dom()
                    })))
                    // Then the static assets
                    .or(serve_dir)
                    // Then all other routes
                    .or(render_ssr(cfg, move || build_virtual_dom()))
            };
            warp::serve(router.boxed().with(warp::filters::compression::gzip()))
                .run(addr)
                .await;
        }
        #[cfg(all(feature = "salvo", not(feature = "axum"), not(feature = "warp")))]
        {
            use crate::adapters::salvo_adapter::{DioxusRouterExt, SSRHandler};
            use salvo::conn::Listener;
            let router = salvo::Router::new().register_server_fns(server_fn_route);
            #[cfg(not(any(feature = "desktop", feature = "mobile")))]
            let router = router
                .serve_static_assets(cfg.assets_path)
                .connect_hot_reload()
                .push(salvo::Router::with_path("/<**any_path>").get(SSRHandler::new(cfg)));
            let router = router.hoop(
                salvo::compression::Compression::new()
                    .enable_gzip(salvo::prelude::CompressionLevel::Default),
            );
            salvo::Server::new(salvo::conn::tcp::TcpListener::new(addr).bind().await)
                .serve(router)
                .await;
        }
