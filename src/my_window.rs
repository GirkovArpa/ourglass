#![macro_use]
use winsafe::{
  co, 
  gui,
  shell,
  CoCreateInstance,
  HINSTANCE, 
  RECT, 
  PAINTSTRUCT, 
  POINT, 
  SIZE, 
  WinResult, 
  COLORREF, 
  HBRUSH,
  IdIdiStr,
  QueryPerformanceCounter, 
  QueryPerformanceFrequency
};


use lazy_static::lazy_static; 
use std::sync::Mutex;

const EDIT_HEIGHT: i32 = 24;
const EDIT2_HEIGHT: i32 = 16;

lazy_static! {
  static ref FREQ: Mutex<i64> = Mutex::new(0);
  static ref START: Mutex<i64> = Mutex::new(0);

  static ref FLASH_START: Mutex<u128> = Mutex::new(0);

  static ref ELAPSED_MILLISECONDS: Mutex<i32> = Mutex::new(0);
  static ref TARGET_MILLISECONDS: Mutex<i32> = Mutex::new(1000);

  static ref LAST_TARGET: Mutex<String> = Mutex::new("1 Seconds".to_string());

  static ref IS_PAUSED: Mutex<bool> = Mutex::new(false);
  static ref IS_TICKING: Mutex<bool> = Mutex::new(false);
  static ref IS_HOVERING_LABEL: Mutex<bool> = Mutex::new(false);
  static ref TIME_IS_UP: Mutex<bool> = Mutex::new(false);

  static ref EDIT_OUTLINE: Mutex<RECT> = Mutex::new(RECT { left: 0, top: 0, right: 0, bottom: 0 });
  static ref EDIT2_OUTLINE: Mutex<RECT> = Mutex::new(RECT { left: 0, top: 0, right: 0, bottom: 0 });

  static ref CLIENT_WIDTH: Mutex<i32> = Mutex::new(0);
  static ref CLIENT_HEIGHT: Mutex<i32> = Mutex::new(0);

  static ref THICKNESS: Mutex<i32> = Mutex::new(13);
  static ref SPACE: Mutex<i32> = Mutex::new(20);
  static ref EDIT_WIDTH: Mutex<i32> = Mutex::new(166);
}

#[derive(Clone)]
pub struct MyWindow {
    wnd: gui::WindowMain,
    lbl: gui::Label,
    lbl2: gui::Label,
    edt: gui::Edit,
    edt2: gui::Edit,
    tskbr: shell::ITaskbarList3
}

impl MyWindow {
    pub fn new() -> MyWindow {
      let hinstance = HINSTANCE::GetModuleHandle(None).unwrap();
      let wnd = gui::WindowMain::new(
        gui::WindowMainOpts {
          title: "OurGlass".to_owned(),
          class_icon: hinstance.LoadIcon(IdIdiStr::Id(1)).unwrap(),
          size: SIZE::new(334, 111),
          style: gui::WindowMainOpts::default().style 
          | co::WS::MINIMIZEBOX 
          | co::WS::MAXIMIZEBOX 
          | co::WS::SIZEBOX,
          class_bg_brush: HBRUSH::CreateSolidBrush(COLORREF::new(0xff, 0xff, 0xff)).unwrap(),
          ..Default::default()
        },
      );

      let lbl = gui::Label::new(
        &wnd, 
        gui::LabelOpts {
          text: "Start".to_owned(),
          size: SIZE::new(50, 20),
          position: POINT::new(234 / 2 - 50, 111 / 2),
          label_style: co::SS::NOTIFY | co::SS::CENTER,
          ..Default::default()
        },
      );

      let lbl2 = gui::Label::new(
        &wnd,
        gui::LabelOpts {
          text: "Stop".to_owned(),
          size: SIZE::new(50, 20),
          position: POINT::new(234 / 2 - 50, 111 / 2),
          label_style: co::SS::NOTIFY | co::SS::CENTER,
          ..Default::default()
        }
      );

      let edt2 = gui::Edit::new(
        &wnd,
        gui::EditOpts {
          text: "Click to enter title".to_owned(),
          width: 200,
          position: POINT::new(234 / 2, 111 / 1),
          edit_style: co::ES::CENTER,
          window_ex_style: co::WS_EX::WINDOWEDGE,
          ..Default::default()
        } 
      );

      let edt = gui::Edit::new(
        &wnd,
        gui::EditOpts {
          text: "1 Second".to_owned(),
          width: 200,
          position: POINT::new(234 / 2, 111 / 1),
          edit_style: co::ES::CENTER,
          window_ex_style: co::WS_EX::WINDOWEDGE,
          ..Default::default()
        } 
      );

      winsafe::CoInitializeEx(co::COINIT::MULTITHREADED).unwrap();

      let tskbr: shell::ITaskbarList3 = CoCreateInstance(
        &shell::clsid::TaskbarList,
        None,
        co::CLSCTX::INPROC_SERVER,
      ).unwrap();

      let new_self = Self { wnd, lbl, lbl2, edt, edt2, tskbr };
      new_self.events();
      new_self
    }

    pub fn run(&self) -> WinResult<()> {
        self.wnd.run_main(None)
    }

    fn events(&self) {
      self.wnd.on().wm_create({
        let self2 = self.clone(); 
        move |_params| {
          self2.lbl2.hwnd().ShowWindow(winsafe::co::SW::HIDE);
          1
        }
      });

      self.wnd.on().wm_close({
        let self2 = self.clone();
        move || {
          if *IS_TICKING.lock().unwrap() || *IS_PAUSED.lock().unwrap() {
            let answer = self2.wnd.hwnd().MessageBox(
              "Are you sure you want to close this timer window?",
              "OurGlass",
              winsafe::co::MB::YESNO | winsafe::co::MB::ICONQUESTION,
            ).unwrap();

            if answer == winsafe::co::DLGID::YES {
              self2.wnd.hwnd().DestroyWindow();
            };
          } else {
            self2.wnd.hwnd().DestroyWindow();
          }
        }
      });

      self.wnd.on().wm_timer(1, {
          let self2 = self.clone();
          move || {
            let time_is_up = *TIME_IS_UP.lock().unwrap();
            if time_is_up {
              self2.wnd.hwnd().InvalidateRect(None, false).unwrap();
            }

            let is_ticking = *IS_TICKING.lock().unwrap();
            if !is_ticking {
              return;
            }

            *ELAPSED_MILLISECONDS.lock().unwrap() += get_counter();
            let elapsed_milliseconds = *ELAPSED_MILLISECONDS.lock().unwrap();
            let target_milliseconds = *TARGET_MILLISECONDS.lock().unwrap();

            let percent = ((elapsed_milliseconds as f32 / target_milliseconds as f32) * 100.) as u64;

            self2.tskbr.SetProgressValue(self2.wnd.hwnd(), percent, 100).unwrap();

            if elapsed_milliseconds <= target_milliseconds {

            } else if elapsed_milliseconds >= target_milliseconds && !*TIME_IS_UP.lock().unwrap() {
              *TIME_IS_UP.lock().unwrap() = true;
              self2.edt.set_text("Timer expired").unwrap();
              self2.lbl.set_text("Reset").unwrap();
              self2.lbl2.set_text("Close").unwrap();
              reposition_labels(&self2);
              *IS_TICKING.lock().unwrap() = false;
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::ERROR).unwrap();
              *FLASH_START.lock().unwrap() = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            }

            self2.wnd.hwnd().InvalidateRect(None, false).unwrap();
            start_counter();
          }
      });

      self.wnd.on().wm_size({
        let self2 = self.clone();
        move |params| {
          self2.wnd.hwnd().InvalidateRect(None, true).unwrap();
          let client_area = params.client_area;
          let width = client_area.cx;
          let height = client_area.cy;
          *CLIENT_WIDTH.lock().unwrap() = width;
          *CLIENT_HEIGHT.lock().unwrap() = height;
          *THICKNESS.lock().unwrap() = ((width.min(height) as f32 / 15_f32) as i32).max(13).min(32);
          *SPACE.lock().unwrap() = (*THICKNESS.lock().unwrap() as f32 * 1.55) as i32;
          
          *EDIT_WIDTH.lock().unwrap() = width - (*SPACE.lock().unwrap() * 2 + *THICKNESS.lock().unwrap() * 2);

          reposition_labels(&self2);
          reposition_edits(&self2);
        }
      });

      self.lbl.on().stn_clicked({
        let self2 = self.clone();
        move || {
          let lbl = &self2.lbl;
          let text = match lbl.text().unwrap().as_ref() {
            "Start" => { 
              *TIME_IS_UP.lock().unwrap() = false;
              *IS_TICKING.lock().unwrap() = true;
              let text = &self2.edt.text().unwrap().to_owned();
              let mut parts = text.split_whitespace();
              let num_str = parts.next().unwrap();
              let num_f32 = num_str.parse::<f32>().unwrap();
              let target_milliseconds = (num_f32 * 1000_f32) as i32;
              *TARGET_MILLISECONDS.lock().unwrap() = target_milliseconds;
              *LAST_TARGET.lock().unwrap() = text.to_string();
              start_counter();
              self2.wnd.hwnd().SetTimer(1, 10, None).unwrap();
              *IS_PAUSED.lock().unwrap() = false;
              self2.lbl2.hwnd().ShowWindow(winsafe::co::SW::SHOW);
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::NORMAL).unwrap();
              "Pause"
            },
            "Pause" => {
              *IS_TICKING.lock().unwrap() = false;
              *IS_PAUSED.lock().unwrap() = true;
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::PAUSED).unwrap();
              "Resume"
            },
            "Resume" => {
              *IS_TICKING.lock().unwrap() = true;
              start_counter();
              *IS_PAUSED.lock().unwrap() = false;
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::NORMAL).unwrap();
              "Pause"
            },
            "Reset" => {
              reset(&self2);
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::NORMAL).unwrap();
              "Start"
            },
            _ => unreachable!()
          };
          self2.lbl.set_text(text).unwrap();
          reposition_labels(&self2);
          self2.lbl.hwnd().InvalidateRect(None, true).unwrap();
        }
      });

      self.lbl.on().stn_dbl_clk({
        let self2 = self.clone();
        move || {
          let lbl = &self2.lbl;
          let text = match lbl.text().unwrap().as_ref() {
            "Start" => { 
              *TIME_IS_UP.lock().unwrap() = false;
              *IS_TICKING.lock().unwrap() = true;
              let text = &self2.edt.text().unwrap().to_owned();
              let mut parts = text.split_whitespace();
              let num_str = parts.next().unwrap();
              let num_f32 = num_str.parse::<f32>().unwrap();
              let target_milliseconds = (num_f32 * 1000_f32) as i32;
              *TARGET_MILLISECONDS.lock().unwrap() = target_milliseconds;
              *LAST_TARGET.lock().unwrap() = text.to_string();
              start_counter();
              self2.wnd.hwnd().SetTimer(1, 10, None).unwrap();
              *IS_PAUSED.lock().unwrap() = false;
              self2.lbl2.hwnd().ShowWindow(winsafe::co::SW::SHOW);
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::NORMAL).unwrap();
              "Pause"
            },
            "Pause" => {
              *IS_TICKING.lock().unwrap() = false;
              *IS_PAUSED.lock().unwrap() = true;
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::PAUSED).unwrap();
              "Resume"
            },
            "Resume" => {
              *IS_TICKING.lock().unwrap() = true;
              start_counter();
              *IS_PAUSED.lock().unwrap() = false;
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::NORMAL).unwrap();
              "Pause"
            },
            "Reset" => {
              reset(&self2);
              self2.tskbr.SetProgressState(self2.wnd.hwnd(), winsafe::shell::co::TBPF::NORMAL).unwrap();
              "Start"
            },
            _ => unreachable!()
          };
          self2.lbl.set_text(text).unwrap();
          reposition_labels(&self2);
          self2.lbl.hwnd().InvalidateRect(None, true).unwrap();
        }
      });

      self.lbl2.on().stn_clicked({
        let self2 = self.clone();
        move || {
          let lbl2 = &self2.lbl2;
          if lbl2.text().unwrap() == "Close" {
            self2.wnd.hwnd().DestroyWindow();
          } else if lbl2.text().unwrap() == "Stop" {
            reset(&self2);
          }
        }
      });

      self.wnd.on().wm_ctl_color_edit({
        let self2 = self.clone();
        move |params| {
          if params.hwnd == self2.edt2.hwnd() {
            let grey_text_color = COLORREF::new(0x80, 0x80, 0x80);
            params.hdc.SetTextColor(grey_text_color).unwrap();
          }
          let white = COLORREF::new(0xff, 0xff, 0xff);
          let white_brush = HBRUSH::CreateSolidBrush(white).unwrap();
          white_brush.DeleteObject().unwrap();
          white_brush
        }
      });

      self.wnd.on().wm_ctl_color_static({
        move |params| { 
          let blue_text_color = COLORREF::new(0x4b, 0x88, 0xc5);
          let red = COLORREF::new(0xff, 0x00, 0x00);
          let white = COLORREF::new(0xff, 0xff, 0xff);
          let red_brush = HBRUSH::CreateSolidBrush(red).unwrap();
          let white_brush = HBRUSH::CreateSolidBrush(white).unwrap();
          params.hdc.SelectObjectBrush(red_brush).unwrap();
          let text_color = if *IS_HOVERING_LABEL.lock().unwrap() { red } else { blue_text_color };
          params.hdc.SetTextColor(text_color).unwrap();
          params.hdc.SetBkMode(winsafe::co::BKMODE::TRANSPARENT).unwrap();

          red_brush.DeleteObject().unwrap();
          white_brush
        }
      });

      self.wnd.on().wm_paint({
        let self2 = self.clone();
        move || {
          let time_is_up = *TIME_IS_UP.lock().unwrap();
          let thickness = *THICKNESS.lock().unwrap();
          let rect = self2.wnd.hwnd().GetClientRect().unwrap();

          let elapsed_milliseconds = *ELAPSED_MILLISECONDS.lock().unwrap() as f32;
          let target_milliseconds = *TARGET_MILLISECONDS.lock().unwrap() as f32;
          let progress_fraction = elapsed_milliseconds / target_milliseconds;
          let progress_width = progress_fraction * rect.right as f32;
          let progress_width = progress_width as i32;

          let mut ps = PAINTSTRUCT::default();
          let hdc = self2.wnd.hwnd().BeginPaint(&mut ps).unwrap();

          let grey = COLORREF::new(0xee, 0xee, 0xee);
          let grey_brush = HBRUSH::CreateSolidBrush(grey).unwrap();

          let (
            top_bar,
            left_bar,
            bottom_bar,
            right_bar
          ) = calculate_grey_bars(rect, progress_width);

          hdc.FillRect(bottom_bar, grey_brush).unwrap();
          hdc.FillRect(top_bar, grey_brush).unwrap();
          hdc.FillRect(left_bar, grey_brush).unwrap();
          hdc.FillRect(right_bar, grey_brush).unwrap();

          let orange_color = COLORREF::new(0xff, 0x7f, 0x50);

          let now: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
                      
          let flash_start = *FLASH_START.lock().unwrap();
          let remaining = now - flash_start;

          let theme_color = if time_is_up && remaining <= 255 * 3 {
            let flash = (now % 255) as u8;
            let flash = ((flash_start + flash as u128) % 255) as u8;
            let fraction = flash as f32 / 255.0;
            lerp_color(String::from("#ff7f50"), String::from("#C65150"), fraction)         
          } else if time_is_up && remaining > 255 * 3 {
            let flash = (now % 4080) as f32;
            let flash = ((flash_start + flash as u128) % 4080) as f32;
            let fraction = flash as f32 / 4080.0;
            let fraction = fraction * 2.0;
            let fraction = if fraction > 1.0 {
              2.0 - fraction
            } else {
              fraction
            };
            lerp_color(String::from("#ff7f50"), String::from("#C65150"), fraction)    
          } else {
            orange_color
          };

          let theme_brush = HBRUSH::CreateSolidBrush(theme_color).unwrap();

          let (
            top_bar, 
            left_bar, 
            bottom_bar, 
            right_bar
          ) = calculate_progress_bars(rect, progress_width);

          hdc.FillRect(bottom_bar, theme_brush).unwrap();
          hdc.FillRect(top_bar, theme_brush).unwrap();
          hdc.FillRect(left_bar, theme_brush).unwrap();
          if progress_width >= rect.right - thickness { hdc.FillRect(right_bar, theme_brush).unwrap(); }

          let (
            top_bar, 
            left_bar, 
            bottom_bar, 
            right_bar
          ) = calculate_inset_bars(thickness, left_bar, top_bar, right_bar, bottom_bar);

          let inset_color = if time_is_up { 
            theme_color
          } else {
            COLORREF::new(0xff, 0xff, 0xff)
          };
          let inset_brush =  HBRUSH::CreateSolidBrush(inset_color).unwrap();
          hdc.FillRect(bottom_bar, inset_brush).unwrap();
          hdc.FillRect(top_bar, inset_brush).unwrap();
          hdc.FillRect(left_bar, inset_brush).unwrap();
          hdc.FillRect(right_bar, inset_brush).unwrap();
          
          let edit_outline = *EDIT_OUTLINE.lock().unwrap();
          let color = COLORREF::new(0xb5, 0xcf, 0xe7);
          let brush = HBRUSH::CreateSolidBrush(color).unwrap();
          hdc.FillRect(edit_outline, brush).unwrap();

          let edit2_outline = *EDIT2_OUTLINE.lock().unwrap();
          hdc.FillRect(edit2_outline, brush).unwrap();

          self2.wnd.hwnd().EndPaint(&ps);

          grey_brush.DeleteObject().unwrap();
          theme_brush.DeleteObject().unwrap();
          brush.DeleteObject().unwrap();
        }
      });
    }
}

fn calculate_grey_bars(client_rect: RECT, progress_width: i32) -> (RECT, RECT, RECT, RECT) {
  let thickness = *THICKNESS.lock().unwrap();
  let progress_width = if *IS_TICKING.lock().unwrap() || *IS_PAUSED.lock().unwrap() {
    progress_width
  } else {
    client_rect.right
  };

  let top_bar = RECT { 
    top: 0, 
    left: progress_width, 
    right: client_rect.right, 
    bottom: thickness 
  };

  let bottom_bar = RECT { 
    top: client_rect.bottom - thickness, 
    left: progress_width, 
    right: client_rect.right, 
    bottom: client_rect.bottom 
  };

  let left_bar = RECT { 
    top: thickness, 
    left: progress_width.min(thickness), 
    right: thickness,
    bottom: client_rect.bottom - thickness 
  };

  let right_bar = RECT { 
    top: thickness, 
    left: client_rect.right - thickness,
    right: (client_rect.right - thickness) + (client_rect.right - progress_width), 
    bottom: client_rect.bottom - thickness 
  };

  (top_bar, left_bar, bottom_bar, right_bar)
}

fn calculate_progress_bars(client_rect: RECT, progress_width: i32) -> (RECT, RECT, RECT, RECT) {
  let thickness = *THICKNESS.lock().unwrap();
  
  let top_bar = RECT { 
    top: 0, 
    left: 0, 
    right: progress_width, 
    bottom: thickness 
  };

  let bottom_bar = RECT {
    top: client_rect.bottom - thickness, 
    left: 0, 
    right: progress_width, 
    bottom: client_rect.bottom 
  };

  let left_bar = RECT { 
    top: thickness, 
    left: 0, 
    right: progress_width.min(thickness), 
    bottom: client_rect.bottom - thickness 
  };

  let right_bar = RECT {
    top: thickness, 
    left: client_rect.right - thickness, 
    right: client_rect.right - (client_rect.right - progress_width),
    bottom: client_rect.bottom - thickness 
  };

  (top_bar, left_bar, bottom_bar, right_bar)
}

fn calculate_inset_bars(thickness: i32, _left_bar: RECT, top_bar: RECT, right_bar: RECT, bottom_bar: RECT) -> (RECT, RECT, RECT, RECT) {
  const SPACE: i32 = 3;
  
  let bottom_bar2 = RECT {
    left: thickness + SPACE,
    top: bottom_bar.top - (SPACE + 1),
    right: right_bar.left - SPACE,
    bottom: bottom_bar.top - SPACE
  };

  let top_bar2 = RECT {
    left: thickness + SPACE,
    top: top_bar.bottom + SPACE,
    right: right_bar.left - SPACE,
    bottom: top_bar.bottom + (SPACE + 1)
  };

  let left_bar2 = RECT {
    left: thickness + SPACE,
    right: thickness + SPACE + 1,
    top: top_bar.bottom + SPACE,
    bottom: bottom_bar.top - SPACE
  };

  let right_bar2 = RECT {
    left: right_bar.left - (SPACE + 1),
    right: right_bar.left - SPACE,
    top: top_bar.bottom + SPACE,
    bottom: bottom_bar.top - SPACE
  };

  (top_bar2, left_bar2, bottom_bar2, right_bar2)
}

fn start_counter() -> () {
  *FREQ.lock().unwrap() = QueryPerformanceFrequency().unwrap();
  *START.lock().unwrap()  = QueryPerformanceCounter().unwrap();
}

fn get_counter() -> i32 {
  (((QueryPerformanceCounter().unwrap() - *START.lock().unwrap()) as f64 / *FREQ.lock().unwrap() as f64) * 1000.0) as i32
}

fn reposition_labels(self2: &MyWindow) -> () {
  let width = *CLIENT_WIDTH.lock().unwrap();
  let height = *CLIENT_HEIGHT.lock().unwrap();
  let center_x = width / 2;
  let center_y = height / 2;

  self2.lbl.hwnd().SetWindowPos(
    winsafe::HwndPlace::Hwnd(self2.wnd.hwnd()), 
    if *IS_PAUSED.lock().unwrap() 
    || *IS_TICKING.lock().unwrap() 
    || *TIME_IS_UP.lock().unwrap() { 
      center_x - 50 
    } else { 
      center_x - 25 
    }, 
    center_y + 17, 
    50, 
    20, 
    winsafe::co::SWP::NOZORDER
  ).unwrap();

  self2.lbl2.hwnd().SetWindowPos(
    winsafe::HwndPlace::Hwnd(self2.wnd.hwnd()), 
    center_x, 
    center_y + 17, 
    50, 
    20, 
    winsafe::co::SWP::NOZORDER
  ).unwrap();

  self2.lbl.hwnd().InvalidateRect(None, true).unwrap();
  self2.lbl2.hwnd().InvalidateRect(None, true).unwrap();
}

fn reposition_edits(self2: &MyWindow) -> () {
  let width = *CLIENT_WIDTH.lock().unwrap();
  let height = *CLIENT_HEIGHT.lock().unwrap();
  let center_x = width / 2;
  let center_y = height / 2;
    
  let edit_width = *EDIT_WIDTH.lock().unwrap();

  self2.edt.hwnd().SetWindowPos(
    winsafe::HwndPlace::Hwnd(self2.wnd.hwnd()), 
    center_x - edit_width / 2, 
    center_y - EDIT_HEIGHT / 2 + 1, 
    edit_width, 
    EDIT_HEIGHT, 
    winsafe::co::SWP::NOZORDER
  ).unwrap();

  self2.edt2.hwnd().SetWindowPos(
    winsafe::HwndPlace::Hwnd(self2.wnd.hwnd()), 
    center_x - edit_width / 2, 
    center_y - EDIT_HEIGHT / 2 + 1 - 19, 
    edit_width, 
    EDIT2_HEIGHT, 
    winsafe::co::SWP::NOZORDER
  ).unwrap();

  let left = center_x - edit_width / 2 - 1;
  let top = center_y - EDIT_HEIGHT / 2;
  let right = left + edit_width + 2;
  let bottom = top + EDIT_HEIGHT + 2;
  let edit_outline = RECT { 
    left: left,
    top: top,
    right: right,
    bottom: bottom
  };

  *EDIT_OUTLINE.lock().unwrap() = edit_outline;

  let left = center_x - edit_width / 2 - 1;
  let top = (center_y - EDIT_HEIGHT / 2 + 1) - 19 - 1;
  let right = left + edit_width + 2;
  let bottom = top + EDIT2_HEIGHT + 2;
  let edit2_outline = RECT { 
    left: left,
    top: top,
    right: right,
    bottom: bottom
  };

  *EDIT2_OUTLINE.lock().unwrap() = edit2_outline;
}

fn reset(self2: &MyWindow) -> () {
  self2.tskbr.SetProgressValue(self2.wnd.hwnd(), 0, 100).unwrap();

  *ELAPSED_MILLISECONDS.lock().unwrap() = 0;
  *TARGET_MILLISECONDS.lock().unwrap() = 0;

  *IS_PAUSED.lock().unwrap() = false;
  *IS_TICKING.lock().unwrap() = false;
  *TIME_IS_UP.lock().unwrap() = false;

  self2.edt.set_text(&*LAST_TARGET.lock().unwrap()).unwrap();
  self2.lbl.set_text("Start").unwrap();
  self2.lbl2.set_text("Stop").unwrap();
  self2.lbl2.hwnd().ShowWindow(winsafe::co::SW::HIDE);

  reposition_labels(&self2);
  self2.wnd.hwnd().InvalidateRect(None, true).unwrap();
}

fn lerp_color(a: String, b: String, fraction: f32) -> COLORREF {
  use bracket_color::prelude::*;

  let a = RGB::from_hex(a).unwrap();
  let b =  RGB::from_hex(b).unwrap();
  let c = a.lerp(b, fraction);

  COLORREF::new((c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8)
}