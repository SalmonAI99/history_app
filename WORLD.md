# 🌐 World Simulator: Ancient Middle East & Canaan World Model Specification

> **Document Version**: 1.0.0  
> **Target System**: Bookreader Ancient Civilization & Middle Eastern History Course (`/simulator`)  
> **Core Framework**: Leptos 0.8 (Rust / WASM) + Axum WebSockets + Three.js WebGL (Isometric 3D)

---

## 📖 1. 概述与核心愿景 (Overview & Vision)

**World Simulator (世界模型模拟器)** 是一个将**古代中东与迦南历史文化课程、圣经考古学实录与多智能体复杂系统 (Multi-Agent Complex Systems)** 融为一体的沉浸式双模态世界模拟平台。

它摆脱了传统静态历史课件的被动单向展示，为学习者提供了：
1. **微观与宏观交织的计算沙盒**：通过智能驾驶模型 (IDM)、基于主体的城市经济模型 (ABM)、有界信任共识算法 (Hegselmann-Krause) 等计算物理与社会学模型，重构公元前 10 世纪古代近东的商道物流、城市运转、旷野生存与哲学论辩。
2. **高精度考古学实景复原**：依据夏琐、米吉多、基色发掘的六室城门、铁器时代四室民居、基训泉引水工程及所罗门圣殿文献记载，建立 10 大具备完整历史考证与实时运转指标的可交互建筑地标。
3. **双重视角自由切换**：支持宏观全息的 **3D 智能体沙盒 (World Simulator)** 与微观个体视角的 **2D 多人联机历史漫游 RPG**。

---

## 🎬 2. 3D 世界模型场景生成提示词规范 (3D World Model Generation Prompt Specification)

> **使用场景**：直接复制以下结构化提示词至 **WorldSimulator.ai**（如 `/create-scenario` 界面）、**Oasis**、**Sora / Runway 3D** 或各种多模态 3D 世界生成器中，精准生成历史考究、细节丰富的迦南城邦世界模型。

### 📋 英语填报模板 (Standard English Prompt for AI World Generators)

```yaml
Title: "The Golden Citadel of Solomon: Ancient Canaanite City-State & Trade Hub (10th Century BCE)"
Category: "Historical Simulation / 3D Isometric World Model / Multi-Agent Complex Systems Sandbox"

Scene Description:
  Visual Style & Camera:
    Isometric orthographic projection (45-degree elevated oblique angle), low-poly stylized realism with physically-based materials: weathered Jerusalem limestone, sun-dried terracotta mudbrick, dark Lebanon cedar timber, burnished reflective bronze, gleaming gold-leaf accents, and translucent turquoise water. Bathed in golden-hour Mediterranean twilight lighting, warm amber sunlight casting soft directional shadows, cool azure ambient rim-light from the evening sky, and gentle atmospheric desert dust haze.

  Setting & Geography:
    A fortified hill-crest Iron Age II citadel set among the dry, rugged arid hills of the Judean highlands, enclosed by defensive perimeter limestone ramparts with crenelated parapets and four corner watchtowers. Running through the southern city gate and connecting to the horizon is the compacted sand King's Highway.

  Architectural Landmarks & Spatial Layout:
    1. Temple Mount (North Acropolis): Elevated limestone terrace. The First Temple (limestone sanctuary hall, gold frieze and cornice, porch) flanked by two colossal bronze pillars (Jachin and Boaz) with lily-shaped capitals. In the outer courtyard, a square tiered bronze altar emits spiraling tendrils of fragrant white-amber incense smoke, beside the Molten Sea basin supported by twelve bronze oxen filled with water.
    2. Royal Palace Complex (West): Palace of the Forest of Lebanon with an imposing colonnade of 45 dark cedarwood tree-trunk pillars, a terraced royal balcony, and deep-purple royal Tyrian fabric banners.
    3. Strategic Granary (Northeast): Three dome-capped cylindrical stone grain silos with conical mudbrick roofs, linked to an enclosed storage depot with stacked burlap grain sacks on wooden ramps.
    4. Gihon Spring Waterworks (Southeast): A sunken stone-lined reservoir carved into bedrock, filled with shimmering translucent cyan spring water, stone descent steps, and a wooden pulley well-crane.
    5. Artisan Quarter (Southwest): A Phoenician bronze foundry with a stone chimney releasing dark wisps of smoke, an open glowing orange-hot furnace forge, anvils, and stacked copper ingots; adjacent is an ancient olive press shed with a circular stone crushing basin, a vertical stone milling wheel turning steadily, and rows of terracotta oil jars (pithoi).
    6. Central Bazaar / Shuk (Center-South): A bustling open-air market plaza shaded by striped fabric awnings in crimson, desert ochre, and indigo, with wooden vendor stalls displaying baskets of frankincense, myrrh, olive oil, and Tyrian purple dyed wool.
    7. Residential Quarter (East): Authentic archaeological Israelite four-room mudbrick houses with central courtyards, stone colonnade pillars, flat timber terrace roofs with safety parapet railings, and drying racks for figs, raisins, and flax.
    8. Six-Chamber Gate & Elders' Seat (South): Monumental Solomonic six-chamber gatehouse with double watchtowers, crenelated battlements, and an inner stone-bench court where white-bearded elders sit in council to administer justice.

Who You Are:
  You are the Royal Chronicler and Overseer of the Realm, commissioned by King Solomon. You possess sovereign oversight to freely navigate every district, inspect strategic grain and water reserves, observe citizen interactions, and adjust economic and religious decrees across the city-state.

Agent Ecosystem & Living Logic:
  The citadel is alive with purposeful autonomous agents:
  - White-robed Levite priests conducting morning and evening offerings at the bronze altar.
  - Phoenician craftsmen hammering bronze ceremonial vessels in the foundry.
  - Local citizens and women carrying earthenware water jars between the four-room houses, the Gihon pool, and the central bazaar.
  - Bedouin spice caravans with single-humped pack camels carrying saddlebags laden with myrrh and purple cloth, queuing at the city gate under Intelligent Driver Model (IDM) physics.
  - Scribes sitting in mudbrick niches carefully reading and copying sacred scriptures onto parchment scrolls.

Additional Notes:
  - Tone & Atmosphere: Reverent, archaeologically authentic, vibrant, and rich with ancient Near Eastern daily life.
  - Interactive Mechanics: Every building and citizen is clickable to reveal deep historical context, live metrics (Grain Storage, Water Level, Treasury Shekels, Sacred Aroma, Citizen Morale), and micro-action triggers.
  - Ambient Motion: Gentle drifting altar smoke, rotating olive millstone, fluttering market canopies, and continuous agent routine pathfinding.

Story Keypoints (Narrative Arc):
  1. Dawn at the Gate: Morning shofar horn sounds; the six-chamber gate opens to admit the incoming Phoenician trade caravan.
  2. The Royal Delivery: Cedarwood logs from Hiram of Tyre are received at the Palace of the Forest of Lebanon.
  3. The Morning Tamid: Levite priests ignite the holy incense on the bronze altar, sending fragrant smoke over the acropolis.
  4. Sluice Gates of Gihon: The subterranean canal feeds fresh spring water into the central cistern to prepare for the dry season.
  5. The Market Balance: Merchants in the central bazaar reach consensus on frankincense prices using official stone balance weights.
  6. Judgment at the Gate: The city elders resolve a property redemption covenant before witnesses on the gatehouse stone benches.
  7. Pressing the First Fruits: The olive stone mill completes cold-pressing pure oil for the golden menorah lamps.
  8. Twilight Over Zion: The setting sun paints the limestone walls in deep amber as oil lamps flicker across the four-room terrace roofs.
```

---

### 🇨🇳 中文生成填报指南 (Chinese Field-by-Field Breakdown)

| 填报字段 (Field) | 建议填入内容 (Recommended Value) |
| :--- | :--- |
| **标题 (Title)** | `所罗门黄金卫城：公元前10世纪古代迦南圣殿城邦与商贸中心` |
| **分类 (Category)** | `历史模拟 / 3D等距世界模型 / 多智能体复杂系统沙盒` |
| **视角与画风 (Style)** | `45度等距俯视正交投影 (Isometric 3D)，低多边形写实主义 (Low-Poly Realism)，地中海傍晚黄昏暖阳，软阴影与天际冷光，耶路撒冷石灰石与夯土色调` |
| **场景环境描述 (Scene Description)** | `一座坐落在犹太山地丘陵之巅的铁器时代II期迦南设防城邦。北部圣殿山耸立着所罗门第一圣殿，门前立有雅斤与波阿斯两根青铜立柱，铜坛上青烟袅袅，旁有十二铜牛铸海；西部为多柱利巴嫩林宫，挂有推罗紫布条幅；东北为三座圆顶石砌粮仓与谷物筒仓；东南为凿石而建的基训泉碧蓝地下蓄水池；西南为推罗青铜高炉冶炼坊与带有旋转石磨的橄榄油坊；中部为各色彩篷密布的商贸大集市；东部为经典的四室泥砖平顶民居群；南部为雄伟的六室防御城门与长老审判法庭，连接着尘土飞扬的王道商路。` |
| **扮演角色 (Who You Are)** | `所罗门王廷特派的王国总督与历史纪事官，拥有视察城邦各街区、调配战略储备、颁布律令与检阅各阶层城民生活的全权视角。` |
| **生态智能体 (Agents)** | `身穿白袍金带的利未祭司、打铁铸铜的推罗工匠、顶陶罐挑水的城邦妇女、在城门按IDM物理排队入城的香料骆驼商队、在阴凉处抄写羊皮纸的学者。` |
| **补充说明 (Notes)** | `庄严神圣、充满古代近东烟火气息。场景内所有建筑物均可点击交互，支持实时粮储、水源、金库、香气与民心等社会学指标动态闭环计算。` |

---

## 🏛️ 3. 系统整体架构 (System Architecture)

```mermaid
graph TD
    Client[Browser Frontend - Leptos WASM] --> Router["/simulator View Controller (src/simulator/view.rs)"]
    
    Router --> Mode1["🌐 3D 智能体沙盒 (Three.js WebGL)"]
    Router --> Mode2["🗺️ 2D 历史漫游 RPG (Rust/WASM 2D Canvas)"]
    
    subgraph "3D World Simulator Engine (public/simulator/sandbox.js)"
        Mode1 --> Raycaster[Raycaster 射线拾取 & 交互]
        Raycaster --> GroundRing[金黄地面聚焦光圈]
        Raycaster --> InspectorHUD[历史个体 & 建筑档案抽屉]
        
        Mode1 --> SubSim1["🐪 古代商道与香料驼队 (IDM物流)"]
        Mode1 --> SubSim2["🏛️ 圣殿城邦与繁荣 (城邦ABM & 10大历史建筑)"]
        Mode1 --> SubSim3["📜 旷野迁徙与抄经使 (集群协作)"]
        Mode1 --> SubSim4["🕊️ 智者论辩与思想派系 (回音室图谱)"]
        
        Mode1 --> AudioSynth[Web Audio 原生合成音效器]
        Mode1 --> ToastUI[浮动操作反馈通知]
    end
    
    subgraph "2D Multiplayer RPG Engine (src/simulator/client.rs & server.rs)"
        Mode2 --> AxumWS[Axum WebSocket Hub (src/simulator/server.rs)]
        Mode2 --> CanvasLoop[2D Canvas 游戏循环 & 碰撞检测]
        Mode2 --> Quests[历史试炼问答 & 升级机制]
    end
```

---

## 🏜️ 3. 3D 智能体沙盒四大模拟子模型 (The Four Simulation Models)

### 模型 1：🐪 古代商道与香料驼队 (Ancient Trade Caravan Network - IDM Logistics)

* **历史背景**：重现古代横贯外约旦与黎凡特的交通咽喉——**王道 (The King's Highway)** 与**香料之路 (Incense Route)** 的贸易交汇口。
* **物理模型：智能驾驶模型 (Intelligent Driver Model, IDM)**
  每个驼队智能体遵循非线性跟驰加速度方程：
  $$\frac{dv}{dt} = a \left[ 1 - \left(\frac{v}{v_0}\right)^\delta - \left(\frac{s^*(v, \Delta v)}{s}\right)^2 \right]$$
  其中期望车距 $s^*$ 考虑了前方骆驼间隙与减速舒适度：
  $$s^*(v, \Delta v) = s_0 + v T + \frac{v \Delta v}{2 \sqrt{a b}}$$
* **瓶颈与税卡**：
  * **古代关隘税卡 (City Gate Checkpoint)**：模拟城门盘查周期 ($2.0s \sim 8.0s$)，引发驼队在城墙入口处形成排队、等待波与放行通流。
* **历史商贸物资**：
  * **黎巴嫩雪松木** (用于圣殿与宫殿建造的大型木料)
  * **阿拉伯没药与乳香** (用于圣所香膏与燔祭圣香)
  * **推罗紫布** (腓尼基名贵染织品，王室与祭司华服)
  * **纯净橄榄油** (金灯台圣油与地中海核心出口物)
  * **大麦与小麦** (耶斯列谷地粮食)
  * **塞浦路斯铜器与陶器** (日常与礼仪用具)

---

### 模型 2：🏛️ 耶路撒冷圣殿城邦与繁荣 (Realistic City-State ABM & Landmarks)

基于圣经考古学 (Biblical Archaeology) 铁器时代 II 期城邦实证，城邦建立在地势起伏的阶梯山地上，外围由坚固环城石墙与四角戍楼保护。

#### 10 大历史建筑地标与考古档案

| 建筑地标 | 归属区域 | 考古与讲义背景 | 实时指标 (Live Stats) | 专属指令交互 |
| :--- | :--- | :--- | :--- | :--- |
| **🏛️ 所罗门第一圣殿 (The First Temple)** | 圣殿山圣所核心区 | 公元前10世纪建于摩利亚山，历时7年由推罗与希伯来工匠共建。殿内以黎巴嫩雪松木包纯金，内设约柜。门前立有**雅斤**（“祂必建立”）与**波阿斯**（“力量在祂”）两根青铜圣柱、大祭司燔祭铜坛（升腾袅袅青烟）与十二铜牛铸海。 | 金库舍客勒、圣香奉纳百分比、每日晨昏二番祭祀 | `🔥 举行晨昏燔祭`<br/>*(敬虔度+15, 金库+25 舍客勒)* |
| **👑 利巴嫩林宫 (Palace of the Forest of Lebanon)** | 王家行政官邸区 | 所罗门历时13年建成的宏伟行政大殿，以45根黎巴嫩雪松为梁柱如同一片森林，内设象牙金宝座，接见推罗、示巴女王等列国使臣，统领十二行政官区。 | 皇家卫队戍卫中、外交朝贡通达 | `📜 颁布王室律令`<br/>*(金库+40, 贸易繁荣+0.15x)* |
| **🌾 皇家战略粮仓与筒仓 (Royal Granary & Silos)** | 王家仓廪储备区 | 应对迦南干旱、蝗灾与围城的生命线。三座圆柱形石砌储粮筒仓与长条形仓库，集中储备来自耶斯列谷地的大麦、小麦与豆类，支撑常备军与利未祭司供养。 | 战略储麦 (公吨)、防灾缓冲周期 | `🌾 开仓赈济分粮`<br/>*(战略储粮-15t, 民心满意+10%)* |
| **💧 基训泉中央蓄水工程 (Gihon Spring Cistern)** | 城邦水利生命线 | 迦南夏季干旱无雨，基训泉是唯一常年自然活水。古代开凿地下暗渠与坚固石砌蓄水池，将城外泉水隐秘引至城内，确保围城不绝水。 | 水池水位百分比、每日市民汲水量 | `💧 开启地下暗渠注水`<br/>*(立即蓄满至 100%)* |
| **⚒️ 推罗青铜工匠冶炼坊 (Phoenician Bronze Foundry)** | 手工业金属区 | 推罗王希兰派遣精通铜铁金银的腓尼基大师户兰（Hiram）在此督工。建有高炉（散发橙红炉火与排烟）、铁砧与堆叠铜锭，铸造圣殿青铜圣器。 | 高炉1080℃、青铜金盘与香炉铸造中 | `⚒️ 锻造精炼青铜礼器`<br/>*(工商业繁荣度+0.1x)* |
| **🫒 古迦南橄榄石磨压榨坊 (Olive Press / Beit Habad)** | 农业深加工区 | 迦南土地三大丰产标志之一（五谷、新酒、纯油）。拥有**动态旋转研磨石轮**与石槽，特级初熟冷榨纯油专供金灯台长明不灭与受膏。 | 初熟圣油产量 (瓮)、石轮运转状态 | `🫒 研磨冷榨特级纯油`<br/>*(圣所香气+8%)* |
| **🏺 中央彩篷大集市 (Central Bazaar & Shuk)** | 商贸贸易核心区 | 汇聚红海阿卡巴湾远洋金砂、阿拉伯乳香没药、推罗紫布与大马士革钢剑，设有条纹彩篷货摊、香料篓与紫袍布卷，市民用标准天平称量银舍客勒交易。 | 贸易繁荣度、今日交易流水 (舍客勒) | `💰 举办列国贸易大集`<br/>*(交易流水+65 舍客勒)* |
| **🏡 铁器时代四室泥砖宅邸群 (Israelite Four-Room House)** | 城民居住生活区 | 圣经考古学铁器时代最典型民居形制。底层中央为露天庭院与牲畜圈，两侧长室储粮；二层为全家卧室；平顶露台（依据《申命记》22:8修筑安全护栏）可晾晒大麦并用于夜间祈祷。 | 约瑟支派工匠家族世系、民心指数 | `🕊️ 视察民间生活`<br/>*(民心满意度+5%)* |
| **⚖️ 六室防御城门与长老法庭 (Six-Chamber City Gate & Elders' Seat)** | 防务关隘与司法区 | 夏琐、米吉多与基色发掘的典型所罗门六室门。古代城门不仅是防卫咽喉，更是**最重要的司法法庭与契约见证地**（如《路得记》波阿斯在此公证赎买土地），门内设有石座与值守长老。 | 城防卫队巡逻中、今日长老公证案件数 | `🛡️ 检阅城防卫队`<br/>*(民心安宁+6%)* |
| **🏰 环城石墙与四角戍楼 (Citadel Fortifications)** | 城邦防御圈 | 坚固石砌环城墙与四座四角瞭望箭楼，守卫城邦通衢要道。 | 城防完好度 100% | 卫城常备戒备状态 |

#### 城民日常昼夜作息与自主行为 (Citizen AI & Daily Routines)
城民根据社会角色与信仰律法拥有真实的日程表：
* **利未祭司**：圣殿祈祷 $\rightarrow$ 铜坛主持晨昏燔祭（圣所香气提升） $\rightarrow$ 视察皇家粮仓什一供奉 $\rightarrow$ 返回圣所。
* **推罗工匠**：冶炼坊打铁锻造 $\rightarrow$ 挑运青铜圣器至圣殿修缮 $\rightarrow$ 中央集市采购物资 $\rightarrow$ 返回工坊。
* **城民平民与妇女**：四室民居启程 $\rightarrow$ 基训泉挑水 $\rightarrow$ 中央集市购买面粉与食油 $\rightarrow$ 圣殿门前献祭祈福 $\rightarrow$ 归家。
* **远洋商贾**：由六室城门牵引驼队入城 $\rightarrow$ 中央集市卸货称银交易 $\rightarrow$ 基训泉汲水补给 $\rightarrow$ 城门出关。

---

### 模型 3：📜 旷野迁徙与典籍传抄守望者 (Desert Migration & Sacred Scroll Custodians)

* **历史背景**：西奈半岛沙漠迁徙、旷野会幕营地与神圣经卷（羊皮纸卷）的守望传承。
* **群体逻辑与生态循环**：
  * **有限耐力与水分消耗模型**：
    $$W(t) = W(0) - \mu_{\text{drain}} \cdot t$$
  * 守经学者手捧羊皮纸经卷在各帐篷间游弋传抄诵读；
  * 当甘泉耐力 $W(t) < 20\%$ 时，个体触发渴水寻路行为，自主返回中央**清泉甘霖 (Oasis Well)** 汲水补给至 $100\%$，随后重返传抄使命，形成可持续的文明传承闭环。

---

### 模型 4：🕊️ 古典智者辩论与思想派系 (Wisdom Debates & Echo Chambers)

* **历史背景**：古典希伯来智慧文学与流散时期的思想流派大辩鸣。
* **四大思想学派**：
  1. **西奈律法学派 (Torah Scholars)**：严谨遵循圣约戒命与正统礼仪（蓝色谱系）。
  2. **所罗门宫廷智者 (Court Sages)**：讲求实用处世箴言、自然哲理与外交智慧（金黄谱系）。
  3. **旷野隐修苦行派 (Desert Ascetics)**：追求沙漠清修、精神纯洁与末世审判预言（翠绿谱系）。
  4. **流散和平博爱先驱 (Peace Advocates)**：倡导普世关怀、宽容与列国和平（洋红谱系）。
* **数学模型：Hegselmann-Krause (HK) 有界信任共识动力学**
  每个智者个体 $i$ 的观点 $x_i \in [0, 1]$ 依有界信任半径 $\epsilon$ 演化：
  $$x_i(t+1) = \frac{1}{|I(i, x)|} \sum_{j \in I(i, x)} x_j(t), \quad I(i, x) = \{j : |x_i(t) - x_j(t)| \le \epsilon \}$$
  当包容阈值 $\epsilon$ 较小时，系统自发分化为互不妥协的回音室激化集群；当 $\epsilon$ 增大时，不同派系逐渐消除偏见，达成普世共识。

---

## 🗺️ 4. 2D 历史漫游多人 RPG (2D Historical Odyssey RPG)

当学习者希望以角色扮演视角亲历文明发展史时，可在顶部切换为 2D 模式：

1. **历史篇章快速旅行 (Fast Travel Chapters)**：
   * **🇪🇬 1. 西奈圣山营地**：颁布十诫律法，沙漠会幕与圣约成立。
   * **🏛️ 2. 所罗门圣殿**：黄金时代王国繁荣，智慧箴言与宏伟建殿。
   * **🏰 3. 欧洲中世纪城镇**：流散社群、抄经学者与手工业自治。
   * **🕊️ 4. 和平讲坛**：先知普世和平愿景与现代文明对话。
2. **实时多人联机与角色职业**：
   * 基于 Axum WebSocket 集线器广播各玩家移动、换图与升级数据；
   * 玩家职业体系：**学者 (Scholar)**、**探险家 (Explorer)**、**商贾 (Merchant)**，各具专属职业增益加成。
3. **课件试炼与对话问答**：
   * 靠近场景中的历史人物按 `[E]` 键触发历史试炼；
   * 答题领悟获得经验升级，触发合成音效与等级头顶光环。

---

## 🎮 5. 交互控制与外部 API (Interaction & Telemetry API)

### 射线拾取与视觉聚光灯
* **鼠标悬停**：当鼠标经过任何可点击建筑或个体时，光标自动变为手形 `pointer`。
* **点击建筑聚焦**：
  * 地面投射出金黄色环形光圈 (`RingGeometry`) 紧扣建筑物地基；
  * 右侧弹出定制的琉璃金边档案卡片，包含历史背景、实时运营指标与可执行指令；
  * 点击空白沙漠区域自动关闭档案与光圈。

### Web Audio 原生音效反馈
无需下载任何外部音频文件，直接调用浏览器 `AudioContext` 实时合成：
* `triggerCitySound('chime')`：圣殿大祭司献祭清脆铜钟和弦。
* `triggerCitySound('horn')`：所罗门王廷角号声。
* `triggerCitySound('water')`：基训泉活水灌满蓄水池涌流声。
* `triggerCitySound('anvil')`：推罗工匠打铁青铜铿锵声。
* `triggerCitySound('coins')`：集市银舍客勒天平称量交割声。

### 运行时控制 API (Window Scope)
```javascript
// 切换四大沙盒模式 ('caravan' | 'temple_city' | 'scribes' | 'wisdom')
window.setSimulationMode(mode);

// 微调参数 (如 'caravan.density', 'templeCity.tradeProsperity', 'wisdom.toleranceThreshold')
window.setSandboxParam(key, value);

// 播放 / 暂停
window.toggleSandboxPlayback();

// 播放倍速 (1.0, 2.0, 5.0)
window.setSandboxSpeed(speedMultiplier);

// 触发建筑微观指令 ('temple_solomon', 'royal_granary', 'gihon_cistern', etc.)
window.triggerBuildingAction(buildingId);

// 🏛️ 地标巡航快速导览：平滑运镜至指定地标并呼出档案
window.focusLandmark(landmarkId);

// 🌅 实时昼夜与光影切换 ('dawn' | 'noon' | 'dusk' | 'night')
// 星夜模式下自动点亮所罗门圣殿铜坛、雅斤波阿斯青铜柱、林宫王廷及六室城门的动态跳跃烛火
window.setTimeOfDay('dawn' | 'noon' | 'dusk' | 'night');

// ⚡ 历史大事件触发 ('caravan_arrival' | 'divine_rain' | 'jubilee_feast' | 'year_of_abundance')
window.triggerHistoricalEvent(eventType);

// 🎥 3D 个体追焦镜头：平滑锁定并跟随当前被点击选中的城民或骆驼
window.toggleAgentFollow();

// 👥 市民招募笔刷：开启后直接点击地面召集并生成新城民智能体
window.toggleSpawnTool();

// 关闭检视卡片
window.closeInspector();
```

---

## 🧭 6. 3D 高级交互组件规范 (Advanced Interactive Components)

为提升历史课件探索性，世界模型提供 6 项高维度交互功能组件：

1. **🏛️ 地标巡航导航条 (Landmark Fast-Travel Ribbon)**
   * 置顶快速导览栏，涵盖第一圣殿、林宫王廷、战略粮仓、基训水池、彩篷集市、青铜工坊、橄榄石磨、四室民居、六室城门。
   * 点击即触发平滑运镜算法 (`controls.target.lerp`)，将 3D 摄像机优雅平移至建筑特写角度，并在建筑脚下展开金黄光环。

2. **🌅 昼夜光照与油灯火光系统 (Day/Night Weather Cycler)**
   * 支持 **黎明 (Dawn)**、**正午 (Noon)**、**黄昏 (Dusk)** 与 **星夜 (Night)** 自由切换。
   * 动态平滑渐变阳光仰角、环境色温与天空背景。
   * **星夜氛围引擎**：进入星夜模式时，自动激活铜坛、雅斤与波阿斯青铜柱、林宫露台与六室城门处布置的历史油灯点光源 (`THREE.PointLight`)，并通过正弦波调制实现逼真的跳跃火光拟真。

3. **⚡ 历史大事件实时触发器 (Historical Event Engine)**
   * **🐫 商队抵城 (Caravan Arrival)**：自王道公路南端即时生成腓尼基进贡商队，按 IDM 跟驰逻辑排队涌向六室城门。
   * **🌧️ 祈降甘霖 (Divine Rain)**：开启三维垂直降落雨滴粒子流 (`THREE.Points`)，滋润迦南大地，使基训地下蓄水池瞬时回满，市民满意度显著提升。
   * **🕊️ 禧年节期 (Jubilee Feast)**：宣告利未记第 25 章神圣禧年，免除债务，全城民心与敬虔度爆满，伴随庄严和弦回荡。
   * **🌾 丰收之年 (Year of Abundance)**：耶斯列谷地五谷丰登，战略粮仓储麦暴增，中央集市交易繁荣。

4. **🎥 3D 个体追焦与第三人称随行 (Agent Follow Camera)**
   * 点击任意移动的利未祭司、推罗工匠、打水妇女或骆驼驼队后，点击检视面板中的 `[🎥 镜头追焦]`。
   * 3D 摄像机目标点将实时绑定智能体位置坐标，跟随其穿梭于四室民居、基训泉与圣殿台阶之间，体验第一线古代生活。

5. **👥 市民招募交互笔刷 (Citizen Spawner Brush)**
   * 点击 `[👥 招募新城民]` 激活地形部署模式。
   * 鼠标点击城邦内任一石质地面，即可就地生成新市民，并依据其社会身份分配四室民居归属与作息轨迹。

6. **📡 战术雷达小地图 (Real-time Tactical Minimap Radar)**
   * 界面左下角圆形雷达（110×110 像素 Canvas，60 FPS 极低开销绘制）。
   * 实时雷达扫描城邦边界、防御石墙与 9 大建筑地标占地脚印。
   * 采用色谱区分智能体移动光点：
     * 🟡 金色：利未祭司
     * 🟠 橙色：推罗工匠
     * 🔵 浅蓝：打水平民与妇女
     * 🟤 棕褐：商贸驼队
     * 🟣 紫色：王室卫队与使者
   * 点击雷达直接响应场景空间感知。

---

## 📁 7. 核心工程文件索引 (Codebase Reference)

* **[public/simulator/sandbox.js](file:///home/thomas/Projects/web/bookreader/public/simulator/sandbox.js)**：3D 智能体沙盒完整实现（Three.js WebGL、四大古代模型、10大可交互建筑地标、IDM跟驰、HK共识、Web Audio 音效合成器）。
* **[src/simulator/view.rs](file:///home/thomas/Projects/web/bookreader/src/simulator/view.rs)**：Leptos 页面视图层，处理 3D 沙盒与 2D RPG 切换、遥测看板、微调滑块与检视窗口 DOM。
* **[src/simulator/client.rs](file:///home/thomas/Projects/web/bookreader/src/simulator/client.rs)**：2D 历史漫游 RPG 客户端引擎（Canvas 循环、输入监听、WebSocket 同步）。
* **[src/simulator/server.rs](file:///home/thomas/Projects/web/bookreader/src/simulator/server.rs)**：Axum 后端 WebSocket 集线器与房间广播通道。
* **[src/simulator/models.rs](file:///home/thomas/Projects/web/bookreader/src/simulator/models.rs)**：智能体状态、玩家职业、客户端/服务端通信协议定义。
