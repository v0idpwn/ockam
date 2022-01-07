use crate::spinner::Spinner;
use crate::AppError;
use comfy_table::Table;
use ockam::{Context, TcpTransport};

pub struct OutletCommand {}

impl OutletCommand {
    pub async fn run(
        ctx: &Context,
        listen: &str,
        name: &str,
        target: &str,
    ) -> Result<(), AppError> {
        let spinner = Spinner::default();

        let tcp = TcpTransport::create(&ctx).await.unwrap();

        tcp.create_outlet(name, target).await.unwrap();

        tcp.listen(listen).await.unwrap();

        spinner.stop("Created outlet");

        let mut table = Table::new();
        table
            .set_header(vec!["Outlet", "Listener", "Destination"])
            .add_row(vec![name, listen, target]);

        println!("{}", table);

        Ok(())
    }
}
