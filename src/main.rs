#![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]
//#![feature(backtrace)]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]

pub mod provider;

#[cfg(all(feature = "qemu", not(esp32)))]
compile_error!("The `qemu` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "ip101", not(esp32)))]
compile_error!("The `ip101` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "kaluga", not(esp32s2)))]
compile_error!("The `kaluga` feature can only be built for the `xtensa-esp32s2-espidf` target.");

#[cfg(all(feature = "ttgo", not(esp32)))]
compile_error!("The `ttgo` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "heltec", not(esp32)))]
compile_error!("The `heltec` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "esp32s3_usb_otg", not(esp32s3)))]
compile_error!(
    "The `esp32s3_usb_otg` feature can only be built for the `xtensa-esp32s3-espidf` target."
);

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
use std::task::Context;
use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};

use anyhow::bail;

use log::*;

use provider::raw::RawTask;
use smol::future::FutureExt;
use url;

use smol;

use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;

use embedded_svc::eth;
use embedded_svc::eth::{Eth, TransitionalState};
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
use embedded_svc::io;
use embedded_svc::ipv4;
use embedded_svc::mqtt::client::{Publish, QoS};
use embedded_svc::ping::Ping;
use embedded_svc::sys_time::SystemTime;
use embedded_svc::timer::TimerService;
use embedded_svc::timer::*;
use embedded_svc::wifi::*;

use esp_idf_svc::eth::*;
use esp_idf_svc::eventloop::*;
use esp_idf_svc::eventloop::*;
use esp_idf_svc::httpd as idf;
use esp_idf_svc::httpd::ServerRegistry;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sntp;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::timer::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::adc;
use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::i2c;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use esp_idf_sys::esp;
use esp_idf_sys::{self, c_types};

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

use ili9341;
use ssd1306;
use ssd1306::mode::DisplayConfig;
use st7789;

use epd_waveshare::{epd4in2::*, graphics::VarDisplay, prelude::*};

#[allow(dead_code)]
#[cfg(not(feature = "qemu"))]
const SSID: &str = "SSID";
#[allow(dead_code)]
#[cfg(not(feature = "qemu"))]
const PASS: &str = "PASS";
static UIN: i64 = 1000i64;
static PASSWORD: &str = "YOUR";
static GROUP_CODE: i64 = 1000i64;
#[cfg(esp32s2)]
include!(env!("EMBUILD_GENERATED_SYMBOLS_FILE"));

#[cfg(esp32s2)]
const ULP: &[u8] = include_bytes!(env!("EMBUILD_GENERATED_BIN_FILE"));

thread_local! {
    static TLS: RefCell<u32> = RefCell::new(13);
}

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    test_print();

    test_atomics();


    esp_idf_svc::log::EspLogger::initialize_default();


    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);



    #[allow(clippy::redundant_clone)]
    #[cfg(not(feature = "qemu"))]
    #[allow(unused_mut)]
    let mut wifi = wifi(
        netif_stack.clone(),
        sys_loop_stack.clone(),
        default_nvs.clone(),
    )?;



    #[cfg(feature = "experimental")]
    experimental::test()?;


    #[cfg(not(feature = "qemu"))]
    {
        drop(wifi);
        info!("Wifi stopped");
    }

    Ok(())
}

#[allow(clippy::vec_init_then_push)]
fn test_print() {
    // Start simple
    println!("Hello from Rust!");

    // Check collections
    let mut children = vec![];

    children.push("foo");
    children.push("bar");
    println!("More complex print {:?}", children);
}

#[allow(deprecated)]
fn test_atomics() {
    let a = AtomicUsize::new(0);
    let v1 = a.compare_and_swap(0, 1, Ordering::SeqCst);
    let v2 = a.swap(2, Ordering::SeqCst);

    let (r1, r2) = unsafe {
        // don't optimize our atomics out
        let r1 = core::ptr::read_volatile(&v1);
        let r2 = core::ptr::read_volatile(&v2);

        (r1, r2)
    };

    println!("Result: {}, {}", r1, r2);
}



#[cfg(feature = "experimental")]
mod experimental {
    use super::{UIN,PASSWORD,GROUP_CODE};
    use super::provider::{
        channel::MyChannelProvider, engine::MyTimeProvider, mutex::MyMutexProvider,
        oneshot::MyOneShotProvider, rwlock::MyRwLockProvider, task::MyTaskProvider,
        tcp::MyTcpStreamSyncProvider,
    };
    use super::{thread, TcpListener, TcpStream};
    use log::info;
    use nrs_qq::msg::MessageChain;
    use nrs_qq::TaskProvider;
    use nrs_qq::{
        device::Device,
        engine::{
            command::wtlogin::{LoginDeviceLocked, LoginResponse, LoginSuccess},
            get_timer_provider, init_random_provider, init_test_random_provider,
            init_timer_provider,
        },
        ext::common::after_login,
        handler::DefaultHandler,
        version::{get_version, Protocol},
        Client, TRwLock, TcpStreamProvider,
    };
    use smol::future::FutureExt;
    use std::sync::Arc;

    use crate::smol::Timer;
    use esp_idf_sys::c_types;
    use futures::StreamExt;
    type MyClient = Client<
        MyChannelProvider,
        MyOneShotProvider,
        MyRwLockProvider,
        MyMutexProvider,
        MyTaskProvider,
        MyTcpStreamSyncProvider,
    >;
    pub fn test() -> anyhow::Result<()> {
        test_tcp_bind_async()?;

        //test_https_client()?;

        Ok(())
    }
    pub fn make_message(msg: String) -> MessageChain {
        use nrs_qq::engine::pb::msg::{elem::Elem as EElem, Text};
        MessageChain::new(vec![EElem::Text(Text {
            str: Some(msg),
            ..Default::default()
        })])
    }
    async fn timer(client: Arc<MyClient>, group_code: i64) {
        let mut interval = Timer::interval(std::time::Duration::from_secs(10));
        std::thread::sleep(std::time::Duration::from_secs(3));
        loop {
            interval.next().await;
            let time = get_timer_provider().now_timestamp_nanos();

            client
                .send_group_message(
                    group_code,
                    make_message(format!("当前时间 {} 来自esp32-c3", time)),
                )
                .await
                .ok();
        }
    }

    async fn app_async() {
        // 这里进行的是拉取时间
        let my_timer = MyTimeProvider::new_form_net_sync("192.168.43.229:7000");
        // 理论上没有拉取时间也行，可以考虑使用下面的代码
        // let my_timer = MyTimeProvider::new();
        info!("my_timer got");
        init_timer_provider(Box::new(my_timer));
        //init_random_provider(Box::new(MyRandomProvidr));
        init_test_random_provider();
        info!("init engine success");
        //let uin = 3536469906i64;
        // let password = std::env::var("PASSWORD").expect("failed to read PASSWORD from env");
        // let password: String = "147852369...".into();
        // let group_code = 528320326i64;
        let my_device: Device = Device {
            display: "GMC.274685.001".into(),
            product: "iarim".into(),
            device: "sagit".into(),
            board: "eomam".into(),
            model: "MI 6".into(),
            finger_print: "xiaomi/iarim/sagit:10/eomam.200122.001/9285333:user/release-keys".into(),
            boot_id: "6fc5a573-d976-013c-20b4-4c00b3a199e1".into(),
            proc_version: "Linux 5.4.0-54-generic-9AKjXfjq (android-build@google.com)".into(),
            imei: "259136341828576".into(),
            brand: "Xiaomi".into(),
            bootloader: "U-boot".into(),
            base_band: "".into(),
            version: nrs_qq::device::OSVersion {
                incremental: "5891938".into(),
                release: "10".into(),
                codename: "REL".into(),
                sdk: 29,
            },
            sim_info: "T-Mobile".into(),
            os_type: "android".into(),
            mac_address: "00:50:56:C0:00:08".into(),
            ip_address: vec![10, 0, 1, 3],
            wifi_bssid: "00:50:56:C0:00:08".into(),
            wifi_ssid: "<unknown ssid>".into(),
            imsi_md5: vec![
                160, 148, 68, 243, 199, 78, 44, 171, 87, 226, 130, 80, 163, 39, 126, 140,
            ],
            android_id: "d8d603f0a7f4d8d2".into(),
            apn: "wifi".into(),
            vendor_name: "MIUI".into(),
            vendor_os_name: "gmc".into(),
        };
        let device = my_device;
        info!("gen device success");

        let client = Arc::new(MyClient::new(
            device,
            get_version(Protocol::IPad),
            DefaultHandler,
        ));
        info!("new client success");
        info!("into block on");
        let stream = MyTcpStreamSyncProvider::connect(client.get_address())
            .await
            .unwrap();
        info!("connect success");
        let c = client.clone();
        let handle = MyTaskProvider::spawn(async move { c.start(stream).await });

        // smol::spawn(async move {
        //     c.start(stream).await;
        // })
        // .detach();
        info!("start success");
        MyTaskProvider::yield_now().await;
        // tracing::info!("准备登录");
        info!("to login");
        let mut resp = match client.password_login(UIN, PASSWORD).await {
            Ok(resp) => resp,
            Err(..) => {
                info!("failed to login with password");
                return;
            }
        };
        loop {
            match resp {
                LoginResponse::Success(LoginSuccess {
                    ref account_info, ..
                }) => {
                    // tracing::info!("login success: {:?}", account_info);
                    info!("login success: {:?}", account_info);
                    break;
                }
                LoginResponse::DeviceLocked(LoginDeviceLocked {
                    ref sms_phone,
                    ref verify_url,
                    ref message,
                    ..
                }) => {
                    // tracing::info!("device locked: {:?}", message);
                    // tracing::info!("sms_phone: {:?}", sms_phone);
                    // tracing::info!("verify_url: {:?}", verify_url);
                    // tracing::info!("手机打开url，处理完成后重启程序");
                    std::process::exit(0);
                    //也可以走短信验证
                    // resp = client.request_sms().await.expect("failed to request sms");
                }
                LoginResponse::DeviceLockLogin { .. } => {
                    resp = client
                        .device_lock_login()
                        .await
                        .expect("failed to login with device lock");
                }
                LoginResponse::AccountFrozen => {
                    log::info!("account frozen");
                }
                LoginResponse::TooManySMSRequest => {
                    log::info!("too many sms request");
                }
                _ => {
                    log::info!("unknown login status: ");
                }
            }
        }
        info!("{:?}", resp);

        after_login(&client).await;
        {
            // client
            //     .reload_friends()
            //     .await
            //     .expect("failed to reload friend list");
            // info!("{:?}", client.friends.read().await);

            // client
            //     .reload_groups()
            //     .await
            //     .expect("failed to reload group list");
            // let group_list = client.groups.read().await;
            // info!("{:?}", group_list);
        }
        {
            // let d = client.get_allowed_clients().await;
            // info!("{:?}", d);
        }
        match client
            .send_group_message(GROUP_CODE, make_message("测试发送 by esp32c3".to_string()))
            .await
        {
            Ok(_) => {
                log::info!("send success");
            }
            Err(e) => {
                log::info!("send error");
            }
        };
        match handle.await {
            Ok(_) => {}
            Err(e) => {
                log::info!("handle error {:?}", e);
            }
        }

        //timer(client, GROUP_CODE).await;
    }


    #[cfg(not(esp_idf_version = "4.3"))]
    fn test_tcp_bind_async() -> anyhow::Result<()> {
        use std::{task::Context, time::Duration};

        use crate::provider::{self, raw::RawTask, task::GLOBAL_EXECUTOR};


        // esp_idf_sys::esp!(unsafe {
        //     esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
        //         max_fds: 5,
        //         ..Default::default()
        //     })
        // })?;


        match thread::Builder::new()
            .stack_size(32 * 1024)
            .name("Main".into())
            .spawn(move || {
                let ex = &GLOBAL_EXECUTOR;
                    ex.spawn(async move {
                        log::info!("app call");
                        app_async().await;
                    })
                    .detach();
                    let mut fu = Box::pin(ex.run(futures::future::pending::<()>()));
                    let task = RawTask::new();
                    loop {
                        match fu.poll(&mut Context::from_waker(&task.to_waker())) {
                            std::task::Poll::Ready(..) => {
                                log::info!("ready");
                                break;
                            }
                            std::task::Poll::Pending => {}
                        }

                        thread::sleep(Duration::from_millis(30))
                    }

            })?
            .join()
        {
            Ok(_) => {}
            Err(e) => {
                log::info!("main thread error");
            }
        }

        Ok(())
    }


}


#[cfg(not(feature = "qemu"))]
#[allow(dead_code)]
fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");

        // ping(&ip_settings)?;
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

