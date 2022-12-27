use std::borrow::Cow;
use std::ops::Range;
use bitfield_struct::bitfield;
use euclid::default::Vector3D;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bird_chat::component::Component;
use bird_chat::identifier::Identifier;
use bird_protocol::{*, ProtocolPacketState::*, ProtocolPacketBound::*};
use bird_protocol::derive::{ProtocolAll, ProtocolPacket};
use bird_util::*;
use crate::nbt::{NbtElement, read_compound_enter, read_named_nbt_tag, write_compound_enter, write_nbt_string};

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
pub struct Slot<'a> {
    #[bp(variant = VarInt)]
    pub item_id: i32,
    pub item_count: i8,
    #[bp(variant = RemainingBytesArray)]
    pub nbt: &'a [u8],
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum HandshakeNextState {
    #[bp(value = 1)]
    Status = 1,
    Login,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x0, state = Handshake, bound = Server)]
pub struct Handshake<'a> {
    #[bp(variant = VarInt)]
    pub protocol_version: i32,
    pub server_address: &'a str,
    pub server_port: u16,
    pub next_state: HandshakeNextState,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseObject<'a> {
    #[serde(borrow)]
    pub version: StatusResponseVersion<'a>,
    #[serde(borrow)]
    pub players: StatusResponsePlayers<'a>,
    #[serde(borrow)]
    pub description: either::Either<&'a str, Component<'a>>,
    #[serde(borrow)]
    pub favicon: Option<&'a str>,
    #[serde(default)]
    pub previews_chat: bool,
    #[serde(default)]
    pub enforces_secure_chat: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct StatusResponseVersion<'a> {
    #[serde(borrow)]
    pub name: &'a str,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StatusResponsePlayers<'a> {
    pub max: i32,
    #[serde(borrow)]
    pub sample: Cow<'a, [StatusResponsePlayersSample<'a>]>,
    pub online: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StatusResponsePlayersSample<'a> {
    #[serde(borrow)]
    pub name: &'a str,
    pub id: Uuid,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x0, state = Status, bound = Client)]
pub struct StatusResponseSS2C<'a>(
    #[bp(variant = Json)]
    pub StatusResponseObject<'a>
);

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1, state = Status, bound = Client)]
pub struct PingResponseSS2C {
    pub payload: u64,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x0, state = Status, bound = Server)]
pub struct StatusRequest;

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1, state = Status, bound = Server)]
pub struct PingRequestSC2S {
    pub payload: u64,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x0, state = Login, bound = Client)]
pub struct LoginDisconnectLS2C<'a> {
    #[bp(variant = Json)]
    pub reason: Component<'a>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1, state = Login, bound = Client)]
pub struct EncryptionRequestLS2C<'a> {
    pub server_id: &'a str,
    #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
    pub public_key: &'a [u8],
    #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
    pub verify_token: &'a [u8],
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
pub struct LoginSuccessProperty<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub signature: Option<&'a str>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x2, state = Login, bound = Client)]
pub struct LoginSuccessLS2C<'a> {
    pub uuid: Uuid,
    pub username: &'a str,
    #[bp(variant = "LengthProvidedArray<i32, VarInt, LoginSuccessProperty<'a>, LoginSuccessProperty<'a>>")]
    pub properties: Cow<'a, [LoginSuccessProperty<'a>]>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x3, state = Login, bound = Client)]
pub struct SetCompressionLS2C {
    #[bp(variant = VarInt)]
    pub threshold: i32,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x4, state = Login, bound = Client)]
pub struct LoginPluginRequestLS2C<'a> {
    #[bp(variant = VarInt)]
    pub message_id: i32,
    pub channel: Identifier<'a>,
    #[bp(variant = RemainingBytesArray)]
    pub data: &'a [u8],
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
pub struct LoginStartSignatureData<'a> {
    pub timestamp: u64,
    #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
    pub public_key: &'a [u8],
    #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
    pub signature: &'a [u8],
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x0, state = Login, bound = Server)]
pub struct LoginStartLC2S<'a> {
    pub name: &'a str,
    pub signature_data: Option<LoginStartSignatureData<'a>>,
    pub uuid: Option<Uuid>,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = bool)]
pub enum EncryptionResponseVariant<'a> {
    #[bp(value = true)]
    VerifyToken {
        #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
        verify_token: &'a [u8]
    },
    #[bp(value = false)]
    Otherwise {
        salt: i64,
        #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
        message_signature: &'a [u8],
    },
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1, state = Login, bound = Server)]
pub struct EncryptionResponseLC2S<'a> {
    #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
    pub shared_secret: &'a [u8],
    pub variant: EncryptionResponseVariant<'a>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x2, state = Login, bound = Server)]
pub struct LoginPluginResponseLC2S<'a> {
    #[bp(variant = VarInt)]
    pub message_id: i32,
    pub successful: bool,
    #[bp(variant = RemainingBytesArray)]
    pub data: &'a [u8],
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x0, state = Play, bound = Client)]
pub struct SpawnEntityPS2C {
    #[bp(variant = VarInt)]
    pub entity_id: i32,
    pub entity_uuid: Uuid,
    #[bp(variant = VarInt)]
    pub entity_type: i32,
    pub position: Vector3D<f64>,
    #[bp(variant = Angle)]
    pub pitch: f32,
    #[bp(variant = Angle)]
    pub yaw: f32,
    #[bp(variant = Angle)]
    pub head_yaw: f32,
    #[bp(variant = VarInt)]
    pub data: i32,
    pub velocity: Vector3D<i16>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1, state = Play, bound = Client)]
pub struct SpawnExperienceOrbPS2C {
    #[bp(variant = VarInt)]
    pub entity_id: i32,
    pub position: Vector3D<f64>,
    pub count: i16,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x2, state = Play, bound = Client)]
pub struct SpawnPlayerPS2C {
    #[bp(variant = VarInt)]
    pub entity_id: i32,
    pub player_uuid: Uuid,
    pub position: Vector3D<f64>,
    #[bp(variant = Angle)]
    pub yaw: f32,
    #[bp(variant = Angle)]
    pub pitch: f32,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = u8)]
pub enum EntityAnimation {
    SwingMainArm,
    TakeDamage,
    LeaveBed,
    SwingOffHand,
    CriticalEffect,
    MagicCriticalEffect,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x3, state = Play, bound = Client)]
pub struct EntityAnimationPS2C {
    #[bp(variant = VarInt)]
    pub entity_id: i32,
    pub animation: EntityAnimation,
}

// Identifies block id in award statistics
pub type AwardStatisticBlock = i32;

// Identified item id in award statistics
pub type AwardStatisticItem = i32;

// Identifier entity id in award statistics
pub type AwardStatisticEntity = i32;

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum AwardStatisticCustom {
    LeaveGame,
    PlayOneMinute,
    TimeSinceDeath,
    TimeSinceRest,
    SneakTime,
    WalkOneCm,
    CrouchOneCm,
    SprintOneCm,
    WalkOnWaterOneCm,
    FallOneCm,
    ClimbOneCm,
    FlyOneCm,
    WalkUnderWaterOneCm,
    MinecartOneCm,
    BoatOneCm,
    PigOneCm,
    HorseOneCm,
    AviateOneCm,
    SwimOneCm,
    StriderOneCm,
    Jump,
    Drop,
    DamageDealt,
    DamageDealtAbsorbed,
    DamageDealtResisted,
    DamageTaken,
    DamageBlockedByShield,
    DamageAbsorbed,
    DamageResisted,
    Deaths,
    MobKills,
    AnimalsBred,
    PlayerKills,
    FishCaught,
    TalkedToVillager,
    TradedWithVillager,
    EatCakeSlice,
    FillCauldron,
    UseCauldron,
    CleanArmor,
    CleanBanner,
    CleanShulkerBox,
    InteractWithBrewingStand,
    InteractWithBeacon,
    InspectDropper,
    InspectHopper,
    InspectDispenser,
    PlayNoteBlock,
    TuneNoteBlock,
    PotFlower,
    TriggerTrappedChest,
    OpenEnderchest,
    EnchantItem,
    PlayRecord,
    InteractWithFurnace,
    InteractWithCraftingTable,
    OpenChest,
    SleepInBed,
    OpenShulkerBox,
    OpenBarrel,
    InteractWithBlastFurnace,
    InteractWithSmoker,
    InteractWithLectern,
    InteractWithCampfire,
    InteractWithCartographyTable,
    InteractWithLoom,
    InteractWithStoneCutter,
    BellRing,
    RaidTrigger,
    RaidWin,
    InteractWithAnvil,
    InteractWithGrindstone,
    TargetHit,
    InteractWithSmithingTable,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum AwardStatistic {
    Mined(
        #[bp(variant = VarInt)]
        AwardStatisticBlock
    ),
    Crafted(
        #[bp(variant = VarInt)]
        AwardStatisticItem
    ),
    Used(
        #[bp(variant = VarInt)]
        AwardStatisticItem
    ),
    Broken(
        #[bp(variant = VarInt)]
        AwardStatisticItem
    ),
    PickedUp(
        #[bp(variant = VarInt)]
        AwardStatisticItem
    ),
    Dropped(
        #[bp(variant = VarInt)]
        AwardStatisticItem
    ),
    Killed(
        #[bp(variant = VarInt)]
        AwardStatisticEntity
    ),
    KilledBy(
        #[bp(variant = VarInt)]
        AwardStatisticEntity
    ),
    Custom(AwardStatisticCustom),
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x4, state = Play, bound = Client)]
pub struct AwardStatisticsPS2C<'a> {
    #[bp(variant = "LengthProvidedArray<i32, VarInt, AwardStatistic, AwardStatistic>")]
    pub statistics: Cow<'a, [AwardStatistic]>,
    #[bp(variant = VarInt)]
    pub value: i32,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x5, state = Play, bound = Client)]
pub struct AcknowledgeBlockChangePS2C {
    #[bp(variant = VarInt)]
    pub sequence_id: i32,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x6, state = Play, bound = Client)]
pub struct SetBlockDestroyStagePS2C {
    #[bp(variant = VarInt)]
    pub entity_id: i32,
    #[bp(variant = BlockPosition)]
    pub location: Vector3D<i32>,
    pub destroy_stage: u8,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x7, state = Play, bound = Client)]
pub struct BlockEntityDataPS2C<'a> {
    #[bp(variant = BlockPosition)]
    pub location: Vector3D<i32>,
    #[bp(variant = VarInt)]
    pub ty: i32,
    #[bp(variant = RemainingBytesArray)]
    pub nbt_data: &'a [u8],
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = u8)]
pub enum BlockActionVariantPistonDirection {
    Down,
    Up,
    South,
    West,
    North,
    East,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = u8)]
pub enum BlockActionVariantBellDirection {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt, key_reverse = true)]
pub enum BlockActionVariant {
    #[bp(value = "(bird_data::block_data::NOTE_BLOCK.id) as i32", ghost = [(order = begin, value = 0u8), (order = end, value = 0u8)])]
    NoteBlock,
    #[bp(value = "(bird_data::block_data::PISTON.id) as i32")]
    Piston {
        retract: bool,
        direction: BlockActionVariantPistonDirection,
    },
    #[bp(value = "(bird_data::block_data::CHEST.id) as i32", ghost = [(order = begin, value = 1u8)])]
    Chest {
        players_looking_in: u8,
    },
    #[bp(value = "(bird_data::block_data::ENDER_CHEST.id) as i32", ghost = [(order = begin, value = 1u8)])]
    EnderChest {
        players_looking_in: u8,
    },
    #[bp(value = "(bird_data::block_data::BEACON.id) as i32", ghost = [(order = begin, value = 1u8), (order = end, value = 0u8)])]
    Beacon,
    #[bp(value = "(bird_data::block_data::SPAWNER.id) as i32", ghost = [(order = begin, value = 1u8), (order = end, value = 0u8)])]
    Spawner,
    #[bp(value = "(bird_data::block_data::END_GATEWAY.id) as i32", ghost = [(order = begin, value = 1u8), (order = end, value = 0u8)])]
    EndGateway,
    #[bp(value = "(bird_data::block_data::SHULKER_BOX.id) as i32", ghost = [(order = begin, value = 1u8)])]
    ShulkerBox {
        players_looking_in: u8,
    },
    #[bp(value = "(bird_data::block_data::BELL.id) as i32", ghost = [(order = begin, value = 1u8)])]
    Bell {
        direction: BlockActionVariantBellDirection,
    },
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x8, state = Play, bound = Client)]
pub struct BlockActionPS2C {
    #[bp(variant = BlockPosition)]
    pub location: Vector3D<i32>,
    pub variant: BlockActionVariant,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x9, state = Play, bound = Client)]
pub struct BlockUpdatePS2C {
    #[bp(variant = BlockPosition)]
    pub location: Vector3D<i32>,
    #[bp(variant = VarInt)]
    pub block_id: i32,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum BossBarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum BossBarDivision {
    Zero,
    Six,
    Ten,
    Twelve,
    Twenty,
}

#[bitfield(u8)]
#[derive(ProtocolAll, PartialEq)]
pub struct BossBarFlags {
    pub dark_sky: bool,
    pub dragon_bar: bool,
    pub fog: bool,
    #[bits(5)]
    _pad: u8,
}

#[derive(ProtocolAll, Clone, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum BossBarAction<'a> {
    Add {
        title: Component<'a>,
        health: f32,
        color: BossBarColor,
        division: BossBarDivision,
        flags: BossBarFlags,
    },
    Remove,
    UpdateHealth {
        health: f32,
    },
    UpdateTitle {
        title: Component<'a>,
    },
    UpdateStyle {
        color: BossBarColor,
        division: BossBarDivision,
    },
    UpdateFlags {
        flags: BossBarFlags,
    },
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0xA, state = Play, bound = Client)]
pub struct BossBarPS2C<'a> {
    pub uuid: Uuid,
    pub action: BossBarAction<'a>,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = u8)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0xB, state = Play, bound = Client)]
pub struct ChangeDifficultyPS2C {
    pub difficulty: Difficulty,
    pub locked: bool,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0xC, state = Play, bound = Client)]
pub struct ChatPreviewPS2C<'a> {
    pub query_id: i32,
    pub message: Option<Component<'a>>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0xD, state = Play, bound = Client)]
pub struct ClearTitles {
    pub reset: bool,
}

#[derive(ProtocolAll, Clone, PartialEq, Debug)]
pub struct CommandSuggestionsMatch<'a> {
    pub insert: &'a str,
    pub tooltip: Option<Component<'a>>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0xE, state = Play, bound = Client)]
pub struct CommandSuggestionsResponsePS2C<'a> {
    #[bp(variant = VarInt)]
    pub id: i32,
    #[bp(variant = VarInt)]
    pub start: i32,
    #[bp(variant = VarInt)]
    pub length: i32,
    #[bp(variant = "LengthProvidedArray<i32, VarInt, CommandSuggestionsMatch<'a>, CommandSuggestionsMatch<'a>>")]
    pub matches: Cow<'a, [CommandSuggestionsMatch<'a>]>,
}

pub const ROOT_NODE_TYPE: u8 = 0;
pub const LITERAL_NODE_TYPE: u8 = 1;
pub const ARGUMENT_NODE_TYPE: u8 = 2;

#[bitfield(i8)]
#[derive(ProtocolAll, PartialEq)]
pub struct BrigadierNodeFlags {
    #[bits(2)]
    pub node_type: u8,
    pub executable: bool,
    pub redirect: bool,
    pub suggestions_type: bool,
    #[bits(3)]
    _pad: u8,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BrigadierNodeRangeProperties<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T> ProtocolSize for BrigadierNodeRangeProperties<T>
    where T: ProtocolSize {
    const SIZE: Range<u32> = add_protocol_sizes_ty!(
        Option<T>,
        Option<T>,
        u8
    );
}

impl<T> ProtocolWritable for BrigadierNodeRangeProperties<T>
    where T: ProtocolWritable {
    fn write<W: ProtocolWriter>(&self, writer: &mut W) -> anyhow::Result<()> {
        let flags = if self.min.is_some() { 1u8 } else { 0u8 } | if self.max.is_some() { 2u8 } else { 0u8 };
        flags.write(writer)?;
        if let Some(ref to_write) = self.min { to_write.write(writer)? };
        if let Some(ref to_write) = self.max { to_write.write(writer)? };
        Ok(())
    }
}

impl<'a, T> ProtocolReadable<'a> for BrigadierNodeRangeProperties<T>
    where T: ProtocolReadable<'a> {
    fn read<C: ProtocolCursor<'a>>(cursor: &mut C) -> ProtocolResult<Self> {
        let flags = u8::read(cursor)?;
        let min = match flags & 0x2 != 0 {
            true => Some(T::read(cursor)?),
            false => None,
        };
        let max = match flags & 0x1 != 0 {
            true => Some(T::read(cursor)?),
            false => None,
        };
        Ok(Self { min, max })
    }
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum BrigadierNodeParserString {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

#[bitfield(u8)]
#[derive(ProtocolAll, PartialEq)]
pub struct BrigadierNodeParseEntity {
    pub single: bool,
    pub only_players: bool,
    #[bits(6)]
    _gap: u8,
}

#[derive(ProtocolAll, Clone, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum BrigadierNodeParser<'a> {
    Bool,
    Float(BrigadierNodeRangeProperties<f32>),
    Double(BrigadierNodeRangeProperties<f64>),
    Integer(BrigadierNodeRangeProperties<i32>),
    Long(BrigadierNodeRangeProperties<i64>),
    String(BrigadierNodeParserString),
    Entity(BrigadierNodeParseEntity),
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
    Component,
    Message,
    Nbt,
    NbtTag,
    NbtPath,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Angle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder {
        multiple: bool,
    },
    Swizzle,
    Team,
    ItemSlot,
    ResourceLocation,
    MobEffect,
    Function,
    EntityAnchor,
    IntRange,
    FloatRange,
    ItemEnchantment,
    EntitySummon,
    Dimension,
    Time,
    ResourceOrTag {
        registry: Identifier<'a>,
    },
    Resource {
        registry: Identifier<'a>,
    },
    TemplateMirror,
    // ?
    TemplateRotation,
    // ?
    Uuid,
}

#[derive(Clone, PartialEq, Debug)]
pub struct BrigadierNode<'a> {
    pub executable: bool,
    pub children: Cow<'a, [i32]>,
    pub redirect_node: Option<i32>,
    pub name: Option<&'a str>,
    pub parser: Option<BrigadierNodeParser<'a>>,
    pub suggestions_type: Option<Identifier<'a>>,
}

impl<'a> ProtocolSize for BrigadierNode<'a> {
    const SIZE: Range<u32> = (
        add_protocol_sizes_ty!(
            u8,
            LengthProvidedArray<i32, VarInt, i32, i32>,
        ).start
            ..
            add_protocol_sizes_ty!(
            u8,
            LengthProvidedArray<i32, VarInt, i32, i32>,
            VarInt,
            &'a str,
            BrigadierNodeParser<'a>,
            Identifier<'a>,
        ).end
    );
}

impl<'a> ProtocolWritable for BrigadierNode<'a> {
    fn write<W: ProtocolWriter>(&self, writer: &mut W) -> anyhow::Result<()> {
        let flags = BrigadierNodeFlags::new()
            .with_node_type(match self.name {
                Some(_) => match self.parser {
                    Some(_) => ARGUMENT_NODE_TYPE,
                    None => LITERAL_NODE_TYPE,
                },
                None => ROOT_NODE_TYPE,
            })
            .with_executable(self.executable)
            .with_redirect(self.redirect_node.is_some())
            .with_suggestions_type(self.suggestions_type.is_some());
        flags.write(writer)?;
        LengthProvidedArray::<i32, VarInt, i32, i32>::write_variant(&self.children, writer)?;
        if let Some(ref to_write) = self.redirect_node { to_write.write(writer)? };
        if let Some(ref to_write) = self.parser { to_write.write(writer)? };
        if let Some(ref to_write) = self.suggestions_type { to_write.write(writer)? };
        Ok(())
    }
}

impl<'a> ProtocolReadable<'a> for BrigadierNode<'a> {
    fn read<C: ProtocolCursor<'a>>(cursor: &mut C) -> ProtocolResult<Self> {
        let flags = BrigadierNodeFlags::read(cursor)?;
        let children = LengthProvidedArray::<i32, VarInt, i32, i32>::read_variant(cursor)?;
        let redirect_node = match flags.redirect() {
            true => Some(VarInt::read_variant(cursor)?),
            false => None,
        };
        let (name, parser) = match flags.node_type() {
            ROOT_NODE_TYPE => (None, None),
            LITERAL_NODE_TYPE => (Some(<&'a str>::read(cursor)?), None),
            _ => (Some(<&'a str>::read(cursor)?), Some(BrigadierNodeParser::read(cursor)?)),
        };
        let suggestions_type = match flags.suggestions_type() {
            true => Some(Identifier::read(cursor)?),
            false => None,
        };
        Ok(Self {
            executable: flags.executable(),
            children,
            redirect_node,
            name,
            parser,
            suggestions_type,
        })
    }
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0xF, state = Play, bound = Client)]
pub struct CommandsPS2C<'a> {
    #[bp(variant = "LengthProvidedArray<i32, VarInt, BrigadierNode<'a>, BrigadierNode<'a>>")]
    pub nodes: Cow<'a, [BrigadierNode<'a>]>,
    #[bp(variant = VarInt)]
    pub root_index: i32,
}

pub const PLAYER_INVENTORY_ID: u8 = 0;

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x10, state = Play, bound = Client)]
pub struct CloseContainerPS2C {
    pub window_id: u8,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x11, state = Play, bound = Client)]
pub struct SetContainerContentPS2C<'a> {
    pub window_id: u8,
    #[bp(variant = VarInt)]
    pub state_id: i32,
    #[bp(variant = "LengthProvidedArray<i32, VarInt, Option<Slot<'a>>, Option<Slot<'a>>>")]
    pub slot_data: Cow<'a, [Option<Slot<'a>>]>,
    pub carried_item: Option<Slot<'a>>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FurnaceProperty {
    FireIcon,
    MaximumFuelBurnTime,
    ProgressArrow,
    MaximumProgress,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnchantmentTableSlot {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnchantmentTableProperty {
    LevelRequirement(EnchantmentTableSlot),
    Seed,
    EnchantmentId(EnchantmentTableSlot),
    EnchantmentLevel(EnchantmentTableSlot),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BeaconProperty {
    PowerLevel,
    FirstPotionEffect,
    SecondPotionEffect,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BrewingStandProperty {
    BrewTime,
    FuelTime,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x12, state = Play, bound = Client)]
pub struct SetContainerPropertyPS2C {
    pub window_id: u8,
    pub property: i16,
    pub value: i16,
}

impl TryFrom<i16> for FurnaceProperty {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FurnaceProperty::FireIcon),
            1 => Ok(FurnaceProperty::MaximumFuelBurnTime),
            2 => Ok(FurnaceProperty::ProgressArrow),
            3 => Ok(FurnaceProperty::MaximumProgress),
            _ => Err(()),
        }
    }
}

impl From<FurnaceProperty> for i16 {
    fn from(value: FurnaceProperty) -> Self {
        match value {
            FurnaceProperty::FireIcon => 0,
            FurnaceProperty::MaximumFuelBurnTime => 1,
            FurnaceProperty::ProgressArrow => 2,
            FurnaceProperty::MaximumProgress => 3,
        }
    }
}

impl TryFrom<i16> for EnchantmentTableProperty {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EnchantmentTableProperty::LevelRequirement(EnchantmentTableSlot::Top)),
            1 => Ok(EnchantmentTableProperty::LevelRequirement(EnchantmentTableSlot::Middle)),
            2 => Ok(EnchantmentTableProperty::LevelRequirement(EnchantmentTableSlot::Bottom)),
            3 => Ok(EnchantmentTableProperty::Seed),
            4 => Ok(EnchantmentTableProperty::EnchantmentId(EnchantmentTableSlot::Top)),
            5 => Ok(EnchantmentTableProperty::EnchantmentId(EnchantmentTableSlot::Middle)),
            6 => Ok(EnchantmentTableProperty::EnchantmentId(EnchantmentTableSlot::Bottom)),
            7 => Ok(EnchantmentTableProperty::EnchantmentLevel(EnchantmentTableSlot::Top)),
            8 => Ok(EnchantmentTableProperty::EnchantmentLevel(EnchantmentTableSlot::Middle)),
            9 => Ok(EnchantmentTableProperty::EnchantmentLevel(EnchantmentTableSlot::Bottom)),
            _ => Err(()),
        }
    }
}

impl From<EnchantmentTableProperty> for i16 {
    fn from(value: EnchantmentTableProperty) -> Self {
        match value {
            EnchantmentTableProperty::LevelRequirement(EnchantmentTableSlot::Top) => 0,
            EnchantmentTableProperty::LevelRequirement(EnchantmentTableSlot::Middle) => 1,
            EnchantmentTableProperty::LevelRequirement(EnchantmentTableSlot::Bottom) => 2,
            EnchantmentTableProperty::Seed => 3,
            EnchantmentTableProperty::EnchantmentId(EnchantmentTableSlot::Top) => 4,
            EnchantmentTableProperty::EnchantmentId(EnchantmentTableSlot::Middle) => 5,
            EnchantmentTableProperty::EnchantmentId(EnchantmentTableSlot::Bottom) => 6,
            EnchantmentTableProperty::EnchantmentLevel(EnchantmentTableSlot::Top) => 7,
            EnchantmentTableProperty::EnchantmentLevel(EnchantmentTableSlot::Middle) => 8,
            EnchantmentTableProperty::EnchantmentLevel(EnchantmentTableSlot::Bottom) => 9,
        }
    }
}

impl TryFrom<i16> for BeaconProperty {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BeaconProperty::PowerLevel),
            1 => Ok(BeaconProperty::FirstPotionEffect),
            2 => Ok(BeaconProperty::SecondPotionEffect),
            _ => Err(()),
        }
    }
}

impl From<BeaconProperty> for i16 {
    fn from(value: BeaconProperty) -> Self {
        match value {
            BeaconProperty::PowerLevel => 0,
            BeaconProperty::FirstPotionEffect => 1,
            BeaconProperty::SecondPotionEffect => 2,
        }
    }
}

impl TryFrom<i16> for BrewingStandProperty {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BrewingStandProperty::BrewTime),
            1 => Ok(BrewingStandProperty::FuelTime),
            _ => Err(()),
        }
    }
}

impl From<BrewingStandProperty> for i16 {
    fn from(value: BrewingStandProperty) -> Self {
        match value {
            BrewingStandProperty::BrewTime => 0,
            BrewingStandProperty::FuelTime => 1,
        }
    }
}

pub const CURSOR_SLOT_ID: i16 = -1;
pub const CURSOR_WINDOW_ID: i8 = -1;

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x13, state = Play, bound = Client)]
pub struct SetContainerSlotPS2C<'a> {
    pub window_id: i8,
    #[bp(variant = VarInt)]
    pub state_id: i32,
    pub slot: i16,
    pub slot_data: Option<Slot<'a>>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x14, state = Play, bound = Client)]
pub struct SetCooldownPS2C {
    #[bp(variant = VarInt)]
    pub item_id: i32,
    #[bp(variant = VarInt)]
    pub cooldown_ticks: i32,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum ChatSuggestionAction {
    Add,
    Remove,
    Set,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x15, state = Play, bound = Client)]
pub struct ChatSuggestionsPS2C<'a> {
    pub action: ChatSuggestionAction,
    #[bp(variant = "LengthProvidedArray<i32, VarInt, &'a str, &'a str>")]
    pub entries: Cow<'a, [&'a str]>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x16, state = Play, bound = Client)]
pub struct PluginMessagePS2C<'a> {
    pub channel: Identifier<'a>,
    #[bp(variant = RemainingBytesArray)]
    pub data: &'a [u8],
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i32, variant = VarInt)]
pub enum CustomSoundCategory {
    Master,
    Music,
    Record,
    Weather,
    Block,
    Hostile,
    Neutral,
    Player,
    Ambient,
    Voice,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x17, state = Play, bound = Client)]
pub struct CustomSoundEffectPS2C<'a> {
    pub sound_name: Identifier<'a>,
    pub sound_category: CustomSoundCategory,
    #[bp(variant = "FixedPointNumber<i32, 3>")]
    pub effect_position_x: f32,
    #[bp(variant = "FixedPointNumber<i32, 3>")]
    pub effect_position_y: f32,
    #[bp(variant = "FixedPointNumber<i32, 3>")]
    pub effect_position_z: f32,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x18, state = Play, bound = Client)]
pub struct HideMessagePS2C<'a> {
    #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
    pub signature: &'a [u8],
}

#[derive(ProtocolAll, ProtocolPacket, Clone, PartialEq, Debug)]
#[bp(id = 0x19, state = Play, bound = Client)]
pub struct DisconnectPS2C<'a> {
    pub reason: Component<'a>,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = i8)]
pub enum EntityEventStatus {
    // TODO
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1A, state = Play, bound = Client)]
pub struct EntityEventPS2C {
    pub entity_id: i32,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1B, state = Play, bound = Client)]
pub struct ExplosionPS2C<'a> {
    pub location: Vector3D<f32>,
    pub strength: f32,
    #[bp(variant = "LengthProvidedRawArray<i32, VarInt, Vector3D<i8>, Vector3D<i8>>")]
    pub records: &'a [Vector3D<i8>],
    pub motion: Vector3D<f32>,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1C, state = Play, bound = Client)]
pub struct UnloadChunkPS2C {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = f32)]
pub enum GameEventGameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = f32)]
pub enum GameEventDemo {
    ShowWelcome,
    #[bp(value = 101f32)]
    TellMovementControls,
    TellJumpControl,
    TellInventoryControl,
    TellDemoIsOver,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = f32)]
pub enum GameEventWinGame {
    RespawnPlayer,
    RollTheCredits,
}

#[derive(ProtocolAll, Clone, Copy, PartialEq, Debug)]
#[bp(ty = f32)]
pub enum GameEventRespawnScreen {
    EnableScreen,
    ImmediatelyRespawn,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1D, state = Play, bound = Client, ty = u8)]
pub enum GameEventPS2C {
    #[bp(ghost = [(order = begin, value = 0f32)])]
    NoRespawnBlockAvailable,
    #[bp(ghost = [(order = begin, value = 0f32)])]
    EndRaining,
    #[bp(ghost = [(order = begin, value = 0f32)])]
    BeginRaining,
    ChangeGameMode(GameEventGameMode),
    WinGame(GameEventWinGame),
    DemoEvent(GameEventDemo),
    #[bp(ghost = [(order = begin, value = 0f32)])]
    ArrowHitPlayer,
    RainLevelChange(f32),
    ThunderLevelChange(f32),
    #[bp(ghost = [(order = begin, value = 0f32)])]
    PufferfishSting,
    #[bp(ghost = [(order = begin, value = 0f32)])]
    ElderGuardianMobAppearance,
    EnableRespawnScreen(GameEventRespawnScreen),
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1E, state = Play, bound = Client)]
pub struct OpenHorseScreenPS2C {
    pub window_id: u8,
    #[bp(variant = VarInt)]
    pub slots: i32,
    pub entity_id: i32,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1E, state = Play, bound = Client)]
pub struct InitializeWorldBorderPS2C {
    pub x: f64,
    pub y: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    #[bp(variant = VarLong)]
    pub speed: i64,
    #[bp(variant = VarInt)]
    pub portal_teleport_boundary: i32,
    #[bp(variant = VarInt)]
    pub warning_blocks: i32,
    #[bp(variant = VarInt)]
    pub warning_seconds: i32,
}

#[derive(ProtocolAll, ProtocolPacket, Clone, Copy, PartialEq, Debug)]
#[bp(id = 0x1F, state = Play, bound = Client)]
pub struct KeepAlivePS2C {
    pub keep_alive_id: i64,
}

#[derive(Clone, Debug)]
pub struct CompactLongsWriter<const BITS: u8> {
    vec: Vec<u64>,
    current: u64,
    current_index: u8,
}

impl<const BITS: u8> CompactLongsWriter<BITS>
    where ConstAssert<{ BITS <= 64 }>: ConstAssertTrue {
    const ELEMENTS_IN_LONG: u8 = 64 / BITS;
    const GAP: u8 = 64 % BITS;

    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            current: 0,
            current_index: 0,
        }
    }

    /// # Safety.
    /// The caller must ensure that the number is not longer than BITS const
    pub unsafe fn push(&mut self, number: u64) {
        debug_assert!(number < (1 << (BITS+1)));
        if self.current_index == Self::ELEMENTS_IN_LONG {
            self.vec.push(self.current);
            self.current = 0;
            self.current_index = 0;
        }
        self.current |= number << (self.current_index * BITS + Self::GAP);
        self.current_index += 1;
    }

    pub fn elements(&self) -> usize {
        self.current_index as usize + (self.vec.len() * (Self::ELEMENTS_IN_LONG as usize))
    }

    pub fn finish(mut self) -> Vec<u64> {
        if self.current_index != 0 {
            self.vec.push(self.current)
        }
        self.vec
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompactLongsReader<I, const BITS: u8, const COUNT: usize> {
    iterator: I,
    current_long: u64,
    next_long: Option<u64>,
    current_index: u8,
}

impl<I: Iterator<Item = u64>, const BITS: u8, const COUNT: usize> CompactLongsReader<I, BITS, COUNT> {
    pub fn new(mut iterator: I) -> Option<Self> {
        let current_long = iterator.next()? >> (64 % BITS);
        let next_long = iterator.next();
        Some(Self {
            iterator,
            current_long,
            next_long,
            current_index: 0,
        })
    }
}

impl<I: Iterator<Item = u64>, const BITS: u8, const COUNT: usize> Iterator for CompactLongsReader<I, BITS, COUNT>
    where ConstAssert<{ BITS <= 64 }>: ConstAssertTrue {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO const evaluation
        if self.next_long.is_none() && self.current_index == {
            let result = COUNT % (64 / BITS as usize);
            if result == 0 { 64 / BITS } else { result as u8 }
        } {
            return None;
        }
        if self.current_index == 64 / BITS {
            self.current_index = 0;
            self.current_long = unsafe { self.next_long.unwrap_unchecked() } >> (64 % BITS);
            self.next_long = self.iterator.next();
        }
        let result = self.current_long & ((1 << BITS) - 1);
        self.current_long >>= BITS;
        self.current_index += 1;
        Some(result)
    }
}

pub const CHUNK_DATA_HEIGHT_MAP_KEY: &'static str = "MOTION_BLOCKING";

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(transparent)]
pub struct ChunkDataHeightMap<'a>(ChunkDataHeightMapInner<'a>);

#[derive(Clone, Copy, PartialEq, Debug)]
#[doc(hidden)]
pub enum ChunkDataHeightMapInner<'a> {
    Raw(&'a [u8]),
    Longs(&'a [u64]),
}

impl<'a> Iterator for ChunkDataHeightMapInner<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Raw(raw) => u64::read(raw).ok(),
            Self::Longs(long) => {
                let number = *long.get(0)?;
                *long = &long[1..];
                Some(number)
            }
        }
    }
}

impl<'a> IntoIterator for ChunkDataHeightMap<'a> {
    type Item = u64;
    type IntoIter = CompactLongsReader<ChunkDataHeightMapInner<'a>, 9, 256>;

    fn into_iter(self) -> Self::IntoIter {
        // SAFETY: It is sure that array of inner struct is not empty.
        unsafe { Self::IntoIter::new(self.0).unwrap_unchecked() }
    }
}

impl<'a> ChunkDataHeightMap<'a> {
    /// # Safety.
    /// The caller must ensure that the length of data slice is 37 * 8
    pub const unsafe fn new_raw(data: &'a [u8]) -> Self {
        debug_assert!(data.len() == 37 * 8);
        Self(ChunkDataHeightMapInner::Raw(data))
    }

    /// # Safety.
    /// The caller must ensure that the length of data is 37
    pub const unsafe fn new_longs(data: &'a [u64]) -> Self {
        debug_assert!(data.len() == 37);
        Self(ChunkDataHeightMapInner::Longs(data))
    }
}

impl<'a> ProtocolSize for ChunkDataHeightMap<'a> {
    const SIZE: Range<u32> = Nbt::SIZE;
}

impl<'a> ProtocolReadable<'a> for ChunkDataHeightMap<'a> {
    fn read<C: ProtocolCursor<'a>>(cursor: &mut C) -> ProtocolResult<Self> {
        read_compound_enter(cursor)?;
        match read_named_nbt_tag(CHUNK_DATA_HEIGHT_MAP_KEY, cursor)? {
            Some(NbtElement::LongArray(data)) => match data.len() == 37 * 8 {
                true => Ok(Self(ChunkDataHeightMapInner::Raw(data))),
                false => Err(ProtocolError::Any(anyhow::Error::msg("MOTION_BLOCKING must be NbtLongArray with exactly 37 length")))
            },
            _ => Err(ProtocolError::Any(anyhow::Error::msg("MOTION_BLOCKING is not NbtLongArray or not present"))),
        }
    }
}

impl<'a> ProtocolWritable for ChunkDataHeightMap<'a> {
    fn write<W: ProtocolWriter>(&self, writer: &mut W) -> anyhow::Result<()> {
        write_compound_enter(writer)?;
        12i8.write(writer)?;
        write_nbt_string(CHUNK_DATA_HEIGHT_MAP_KEY, writer)?;
        match self.0 {
            ChunkDataHeightMapInner::Raw(raw) => {
                37i32.write(writer)?; // the length of raw
                writer.write_bytes(raw)
            }
            ChunkDataHeightMapInner::Longs(array) => LengthProvidedArray::<i32, i32, u64, u64>::write_variant(array, writer)?,
        }
        0i8.write(writer)
    }
}

#[derive(ProtocolAll, Clone, Copy, Debug)]
pub struct ChunkSectionsData<'a> {
    #[bp(variant = "LengthProvidedBytesArray<i32, VarInt>")]
    pub data: &'a [u8],
}

pub struct ChunkSectionData {

}

#[derive(ProtocolAll, Clone, Copy, Debug)]
pub struct ChunkData<'a> {
    pub height_map: ChunkDataHeightMap<'a>,
    pub chunk_sections: ChunkSectionsData<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_longs_reader_test() {
        let mut compact_longs_reader = CompactLongsReader::<_, 9, 19>::new(
            vec![
                0b111111111_001111111_000011111_000000111_000000001_0; 3
            ].into_iter()
        ).unwrap();
        for i in 0..3 {
            assert_eq!(compact_longs_reader.next(), Some(0b1));
            assert_eq!(compact_longs_reader.next(), Some(0b111));
            assert_eq!(compact_longs_reader.next(), Some(0b11111));
            assert_eq!(compact_longs_reader.next(), Some(0b1111111));
            assert_eq!(compact_longs_reader.next(), Some(0b111111111));
            if i == 2 {
                assert_eq!(compact_longs_reader.next(), None);
            }
            else {
                assert_eq!(compact_longs_reader.next(), Some(0b0));
                assert_eq!(compact_longs_reader.next(), Some(0b0));
            }
        }
    }

    #[test]
    fn compact_longs_writer_test() {
        let mut compact_longs_writer = CompactLongsWriter::<9>::new();
        unsafe {
            for i in 0..3 {
                compact_longs_writer.push(0b1);
                compact_longs_writer.push(0b111);
                compact_longs_writer.push(0b11111);
                compact_longs_writer.push(0b1111111);
                compact_longs_writer.push(0b111111111);
                if i != 2 {
                    compact_longs_writer.push(0b0);
                    compact_longs_writer.push(0b0);
                }
            }
        }
        assert_eq!(compact_longs_writer.finish(), vec![0b111111111_001111111_000011111_000000111_000000001_0; 3]);
    }
}
