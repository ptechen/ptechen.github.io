use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use crate::request::{HttpRequest, HttpResponse};
use gloo_storage::Storage;
use yew::platform::spawn_local;
use components::card::Card;
use gloo_console::log;
use components::dropdowns::{DropdownsProps, UlProps, LiProps};
use components::alert::Alert;
use components::list_group::ListGroup;

const URL: &'static str = "/api/teamup/room/search";
const KICK_OUT: &'static str = "/api/teamup/room/kickout?token=oP0dU4YXxjU5vJ1MO3JdAbBhPKwtOEIyz36o2F";

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub onclick: Callback<AttrValue>,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ListTeamUpRoomsRequest {
//     pub game_type: GameType,
//     // 地图ID
//     pub map_id: Option<String>,
//     // 地图类型
//     pub map_category: Option<u8>,
//     pub page_size: u64,
//     pub page_index: u64,
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListTeamUpRoomsRequest {
    // 一级类目
    pub cate1: Option<String>,
    // 二级类目
    pub cate2: Option<String>,
    // 标签
    pub tags: Option<Vec<String>>,
    // 关键字
    pub keywords: Option<String>,
    // 地图ID
    pub map_id: Option<String>,
    // 分页大小
    pub page_size: u64,
    // 页码
    pub page_index: u64,
    /// 活动的游戏类型
    pub activity_game_type: Option<String>,
    // 是否开启观战
    pub stream_observer: Option<bool>,
}

impl Default for ListTeamUpRoomsRequest {
    fn default() -> Self {
        Self {
            cate1: Some("War3".to_string()),
            cate2: None,
            tags: None,
            map_id: None,
            page_size: 1000,
            page_index: 1,
            activity_game_type: None,
            keywords: None,
            stream_observer: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ListTeamUpRoomsResponse {
    // 分页大小
    pub page_size: u64,
    // 页码
    pub page_index: u64,
    // 房间总数
    pub total: u64,
    // 搜索关键字
    pub keywords: Option<String>,
    // 房间列表
    pub rooms: Vec<TeamUpRoom>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamUpRoom {
    // 房间ID
    // Since the room ID is generated by the Redis `INCR` command, and
    // the return value of `INCR` is a signed integer, so we use `i64` to
    // correspond to it here.
    pub id: i64,
    // 房间名称
    pub name: String,
    // 房间号
    pub room_no: String,
    // 是否有房间密码
    pub has_secret: bool,
    // 房间密码
    pub secret: Option<String>,
    // 是否自动踢出不准备玩家
    #[serde(default)]
    pub auto_kick: bool,
    // 房主
    // pub created_by: PlayerInfo,
    // 游戏类型
    pub game_type: GameType,
    // 房间类型
    pub room_type: RoomType,
    // 房间状态
    pub room_state: RoomState,
    // 玩家
    pub players: Vec<PlayerInfo>,
    // // 槽位配置
    // pub slots: Option<Vec<Slot>>,
    // // 游戏配置
    // pub game_config: GameConfig,
    // 房间最大人数
    pub max_players: u8,
    // 已匹配时间(room_state 为 Matching 时有效)
    pub matching_time: Option<i64>,
    // 匹配开始时间(room_state 为 Matching 时有效)
    pub match_start_time: Option<i64>,
    // 匹配结束时间(“取消匹配、匹配成功”时更新)
    pub match_end_time: Option<i64>,
    // 最近一场对局ID
    pub last_game_id: Option<u64>,
    // IM 频道
    // pub im_channel: Option<ImChannelInfo>,
    /// 赛季/活动id
    pub activity_id: Option<u32>,
    // pub activity_params: Option<HashMap<String, String>>,

    /// 最后活跃时间，用于排序房间
    ///
    /// 1. 新创建的房间，“最后活跃时间”即创建的时间
    /// 2. 房主邀请、开始匹配、取消匹配、游戏结束、房主更换槽位 时更新“最后活跃时间”
    /// 3. 玩家离开时，不更新“最后活跃时间”
    #[serde(default)]
    pub last_active_time: i64,

    /// 房间创建时间
    #[serde(default)]
    pub created_at: i64,
    // // // // 地图状态
    // #[serde(default = "default_map_status")]
    // pub map_status: u8,
    // 裁判位
    #[serde(default)]
    pub referees_slots: bool,
    /// None: 展示 Some: 不展示
    pub is_display: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInfo {
    // 用户ID
    pub user_id: String,
    pub nick: Option<String>,
    pub activity_timeout: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RoomType {
    /// 自建房间
    Custom,
    /// 快速匹配
    QuickMatch,
    /// 天梯排位
    Ladder,
    /// 活动
    Activity,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RoomState {
    // 等待人满(部分游戏要求人满才能开始)
    WaitingFullfill,
    // 等待玩家准备
    WaitingReady,
    // 等待开始(所有人都已准备)
    Ready,
    // 匹配中
    Matching,
    // 游戏中
    Gaming,
    // 游戏结束,结算中
    GameOver,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum GameType {
    War3,
    RPG,
    CH3C,
}

pub enum Msg {
    Init(ListTeamUpRoomsResponse),
    KickOut((usize, i64, Vec<PlayerInfo>)),
    UpdateGameType(AttrValue),
    AlertStop(bool),
    Refresh,
}

impl From<&str> for GameType {
    fn from(value: &str) -> Self {
        match value {
            "War3" => Self::War3,
            "RPG" => Self::RPG,
            "CH3C" => Self::CH3C,
            _ => Self::War3
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KickOutRoomsRequest {
    pub user_ids: Vec<String>,
    pub room_id: i64,
}

pub struct RoomList {
    pub params: ListTeamUpRoomsRequest,
    pub res: ListTeamUpRoomsResponse,
    pub dropdowns: DropdownsProps,
    pub alert: bool,
}

impl Component for RoomList {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut game_type: String = gloo_storage::SessionStorage::get("room_list_game_type").unwrap_or_default();
        if game_type.is_empty() {
            game_type = "War3".to_string();
        }
        let mut params = ListTeamUpRoomsRequest::default();
        params.cate1 = Some(game_type.to_owned());
        let onclick = ctx.link().callback(|msg| Msg::UpdateGameType(msg));
        Self {
            params,
            res: ListTeamUpRoomsResponse::default(),
            dropdowns: DropdownsProps {
                title: AttrValue::from(format!("对战类型: {}", game_type)),
                ul: UlProps {
                    lis: vec![
                        LiProps { title: AttrValue::from("War3") },
                        LiProps { title: AttrValue::from("RPG") },
                        LiProps { title: AttrValue::from("CH3C") },
                    ]
                },
                callback: onclick,
            },
            alert: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Init(msg) => {
                self.res = msg;
                true
            }
            Msg::KickOut((index, room_id, msg)) => {
                self.res.rooms.get_mut(index).unwrap().is_display = Some(true);
                Self::send_kick_out(room_id, msg);
                self.alert = true;
                true
            }
            Msg::UpdateGameType(msg) => {
                self.params.cate1 = Some(msg.to_string());
                self.dropdowns.title = AttrValue::from(format!("对战类型: {}", self.params.cate1.clone().unwrap_or_default()));
                gloo_storage::SessionStorage::set("room_list_game_type", self.params.cate1.to_owned().unwrap_or_default()).unwrap_or_default();
                let call_back = ctx.link().callback(|msg| Msg::Init(msg));
                Self::get_room_list(call_back, self.params.clone());
                true
            }
            Msg::AlertStop(msg) => {
                self.alert = msg;
                true
            }
            Msg::Refresh => {
                let call_back = ctx.link().callback(|msg| Msg::Init(msg));
                Self::get_room_list(call_back, self.params.clone());
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let call_back = ctx.link().callback(|msg| Msg::Init(msg));
            Self::get_room_list(call_back, self.params.clone());
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let rooms = self.res.rooms.clone();
        let double_click = ctx.link().callback(|msg| Msg::KickOut(msg));
        let lis = self.room_list(rooms.clone(), double_click);
        let mut alert = html!();
        if self.alert {
            let callback = ctx.link().callback(|msg| Msg::AlertStop(msg));
            alert = html!(<Alert text={"发送成功"} {callback}/>);
        };
        let refresh = ctx.link().callback(|_| Msg::Refresh);
        let dropdowns = self.dropdowns.to_html();
        if !rooms.is_empty() {
            html!(
                <div>
                    <div>
                        {alert}
                    </div>
                    <div>
                    {dropdowns}
                    <button class="btn btn-secondary" onclick={refresh}>{"刷新"}</button>
                    </div>
                <div class="flex-div">
                    <ListGroup {lis}/>
                </div>
                </div>
            )
        } else {
            html!({dropdowns})
        }
    }
}

impl RoomList {
    fn get_room_list(callback: Callback<ListTeamUpRoomsResponse>, params: ListTeamUpRoomsRequest) {
        let host: String = gloo_storage::SessionStorage::get("current_env").unwrap_or_default();
        let params = serde_json::to_string(&params).unwrap_or_default();
        spawn_local(async move {
            let res: HttpResponse<ListTeamUpRoomsResponse> = HttpRequest::post(params, &(host + URL)).await.unwrap();
            let res = res.data.unwrap_or_default();
            callback.emit(res);
        })
    }

    fn room_list(&self, rooms: Vec<TeamUpRoom>, on_dblclick: Callback<(usize, i64, Vec<PlayerInfo>)>) -> Vec<Html> {
        let mut lis = vec![];
        for (index, item) in rooms.iter().enumerate() {
            if item.is_display.is_some() {
                continue;
            }
            if Utc::now().timestamp() - item.last_active_time > 3600 {
                let ondblclick = on_dblclick.clone();
                let room = Self::get_li(ondblclick, index, item, AttrValue::from("btn btn-success"));
                lis.push(html!(<Card html={room}/>));
                // html!(<Li html={card} class={"list-group-item"}/>)
            } else {
                let ondblclick = on_dblclick.clone();
                let room = Self::get_li(ondblclick, index, item, AttrValue::from("btn btn-danger"));
                lis.push(html!(<Card html={room}/>));
            }
        }
        lis
    }

    fn get_li(ondblclick: Callback<(usize, i64, Vec<PlayerInfo>)>, index: usize, item: &TeamUpRoom, button_class: AttrValue) -> Html {
        let players = item.players.clone();
        let players_c = players.clone();
        let room_id = item.id;
        let ondblclick = Callback::from(move |_event: MouseEvent| {
            ondblclick.emit((index, room_id, players_c.clone()));
        });
        html!(
                    <div class="room-info-width">
                    <div class="row">{format!("房间id:{}", item.id.clone())}</div>
                    <div class="row">{format!("房号：{}", item.room_no.clone())}</div>
                    <div class="row">{format!("房间名:{}", item.name.clone())}</div>
                    <div class="row">{format!("房间状态:{:?}", item.room_state.clone())}</div>
                    <div class="row">{format!("活跃时间:{}", NaiveDateTime::from_timestamp_opt(item.last_active_time.clone() + 3600 * 8, 0).unwrap().format("%Y-%m-%d %H:%M:%S").to_string())}</div>
                    <div class="row">{format!("房间玩家数:{:?}", players.len())}</div>
                    if players.is_empty() {
                        <button class={button_class} {ondblclick}>{"警告房间为空"}</button>
                    } else {
                        <button class={button_class} {ondblclick}>{"踢出"}</button>
                    }
                    </div>
                )
    }

    fn send_kick_out(room_id: i64, msg: Vec<PlayerInfo>) {
        let mut user_ids = vec![];
        for player in msg {
            user_ids.push(player.user_id);
        }
        let params = serde_json::to_string(&KickOutRoomsRequest { user_ids, room_id }).unwrap();
        let host: String = gloo_storage::SessionStorage::get("current_env").unwrap_or_default();
        spawn_local(async move {
            if let Err(e) = HttpRequest::post::<String, ()>(params, &(host + KICK_OUT)).await {
                log!("send_kick_out", e.to_string());
            };
        })
    }
}
