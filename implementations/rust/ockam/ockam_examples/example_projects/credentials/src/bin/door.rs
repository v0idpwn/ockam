use lib::{
    read_entity_id, DoorIsOpenedMessage, OpenDoorMessage, DOOR_CONTROLLER_ADDRESS,
    DOOR_LISTENER_ADDRESS, DOOR_TCP_ADDRESS, DOOR_VERIFIER_ADDRESS, OFFICE_LISTENER_ADDRESS,
    OFFICE_TCP_ADDRESS,
};
use ockam::{
    credential_attribute_values, credential_type, get_secure_channel_participant_id, route,
    Context, CredentialProtocol, Entity, IdentifierTrustPolicy, Identity, NoOpTrustPolicy, Result,
    Routed, SecureChannels, TcpTransport, Vault, Verifier1, Worker, TCP,
};

struct DoorController {
    entity: Entity,
}

#[ockam::worker]
impl Worker for DoorController {
    type Message = OpenDoorMessage;
    type Context = Context;

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        let requester_id = get_secure_channel_participant_id(&msg)?;

        let verifier = self.entity.check_remote_credential(
            &requester_id,
            credential_type!["TYPE_ID"; "door_id", (Number, "can_open_door")],
            credential_attribute_values![self.entity.identifier()?.to_string(), 1],
        )?;

        if !verifier {
            return Ok(());
        }

        // Controller triggers actual hardware here

        let route = msg.return_route();
        ctx.send(route, DoorIsOpenedMessage).await?;

        Ok(())
    }
}

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    let vault = Vault::create(&ctx)?;
    let mut entity = Entity::create(&ctx, &vault)?;

    ctx.start_worker(
        DOOR_CONTROLLER_ADDRESS,
        DoorController {
            entity: entity.clone(),
        },
    )
    .await?;

    println!("Door id: {}", entity.identifier()?);

    println!("Enter Office id: ");
    let office_id = read_entity_id()?;

    let tcp = TcpTransport::create(&ctx).await?;
    tcp.connect(OFFICE_TCP_ADDRESS).await?;

    // Just to get office's profile
    let _office_channel = entity.create_secure_channel(
        route![(TCP, OFFICE_TCP_ADDRESS), OFFICE_LISTENER_ADDRESS],
        IdentifierTrustPolicy::new(office_id.clone()),
    )?;

    entity.create_secure_channel_listener(DOOR_LISTENER_ADDRESS, NoOpTrustPolicy {})?;

    tcp.listen(DOOR_TCP_ADDRESS).await?;

    // TODO: Add listener
    let res = entity.create_credential_verifier_listener(
        DOOR_VERIFIER_ADDRESS,
        &office_id,
        credential_type!["TYPE_ID"; "door_id", (Number, "can_open_door")],
    )?;
    assert!(res);

    // TODO: Add credential expiration

    println!("Door is opened!");

    Ok(())
}
