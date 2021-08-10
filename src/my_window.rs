#![macro_use]
use winsafe::{co, gui};
use winsafe::{
  HINSTANCE, 
  RECT, 
  PAINTSTRUCT, 
  POINT, 
  SIZE, 
  WinResult, 
  COLORREF, 
  HBRUSH,
  IdIdiStr
};

use lazy_static::lazy_static; 
use std::sync::Mutex;

const EDIT_WIDTH: i32 = 166;
const EDIT_HEIGHT: i32 = 24;
const EDIT2_HEIGHT: i32 = 16;

lazy_static! {
  static ref ELAPSED_MILLISECONDS: Mutex<i32> = Mutex::new(0);
  static ref TARGET_MILLISECONDS: Mutex<i32> = Mutex::new(2500);
  static ref IS_TICKING: Mutex<bool> = Mutex::new(false);
  static ref IS_HOVERING_LABEL: Mutex<bool> = Mutex::new(false);
  static ref EDIT_OUTLINE: Mutex<RECT> = Mutex::new(RECT { left: 0, top: 0, right: 0, bottom: 0 });
  static ref EDIT2_OUTLINE: Mutex<RECT> = Mutex::new(RECT { left: 0, top: 0, right: 0, bottom: 0 });
  static ref THICKNESS: Mutex<i32> = Mutex::new(13);
}

#[derive(Clone)]
pub struct MyWindow {
    wnd: gui::WindowMain,
    lbl: gui::Label,
    edt: gui::Edit,
    edt2: gui::Edit
}

impl MyWindow {
    pub fn new() -> MyWindow {
      let hinstance = HINSTANCE::GetModuleHandle(None).unwrap();
      let wnd = gui::WindowMain::new(
        gui::WindowMainOpts {
          title: "OurGlass".to_owned(),
          class_icon: hinstance.LoadIcon(IdIdiStr::Id(1)).unwrap(),
          size: SIZE::new(234, 111),
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
        size: SIZE::new(100, 20),
        position: POINT::new(234 / 2 - 50, 111 / 2),
        label_style: co::SS::NOTIFY | co::SS::CENTER,
        ..Default::default()
        },
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
          text: "2.5 Seconds".to_owned(),
          width: 200,
          position: POINT::new(234 / 2, 111 / 1),
          edit_style: co::ES::CENTER,
          window_ex_style: co::WS_EX::WINDOWEDGE,
          ..Default::default()
        } 
      );

      let new_self = Self { wnd, lbl, edt, edt2 };
      new_self.events();
      new_self
    }

    pub fn run(&self) -> WinResult<()> {
        self.wnd.run_main(None)
    }

    fn events(&self) {
      self.wnd.on().wm_timer(1, {
          let self2 = self.clone();
          move || {
            let elapsed_milliseconds = *ELAPSED_MILLISECONDS.lock().unwrap();
            let target_milliseconds = *TARGET_MILLISECONDS.lock().unwrap();
            let is_ticking = *IS_TICKING.lock().unwrap();

            if elapsed_milliseconds < target_milliseconds && is_ticking {
              *ELAPSED_MILLISECONDS.lock().unwrap() += 10;
              self2.wnd.hwnd().InvalidateRect(None, false).unwrap();
            } else if elapsed_milliseconds == target_milliseconds && is_ticking {
              self2.lbl.set_text("Restart").unwrap();
              *IS_TICKING.lock().unwrap() = false;
            }
          }
      });

      self.wnd.on().wm_size({
        let self2 = self.clone();
        move |params| {
          self2.wnd.hwnd().InvalidateRect(None, true).unwrap();
          let client_area = params.client_area;
          let width = client_area.cx;
          let height = client_area.cy;
          *THICKNESS.lock().unwrap() = ((width.min(height) as f32 / 15_f32) as i32).max(13).min(32);
          let x = width / 2;
          let y = height / 2;
          self2.lbl.set_text(&self2.lbl.text().unwrap()).unwrap();
          self2.lbl.hwnd().SetWindowPos(
            winsafe::HwndPlace::Hwnd(self2.wnd.hwnd()), 
            x - 50, 
            y + 17, 
            0, 
            10, 
            winsafe::co::SWP::NOSIZE | winsafe::co::SWP::NOZORDER
          ).unwrap();
          self2.edt.hwnd().SetWindowPos(
            winsafe::HwndPlace::Hwnd(self2.wnd.hwnd()), 
            x - EDIT_WIDTH / 2, 
            y - EDIT_HEIGHT / 2 + 1, 
            EDIT_WIDTH, 
            EDIT_HEIGHT, 
            winsafe::co::SWP::NOZORDER
          ).unwrap();
          self2.edt2.hwnd().SetWindowPos(
            winsafe::HwndPlace::Hwnd(self2.wnd.hwnd()), 
            x - EDIT_WIDTH / 2, 
            y - EDIT_HEIGHT / 2 + 1 - 19, 
            EDIT_WIDTH, 
            EDIT2_HEIGHT, 
            winsafe::co::SWP::NOZORDER
          ).unwrap();
          let left = x - EDIT_WIDTH / 2 - 1;
          let top = y - EDIT_HEIGHT / 2;
          let right = left + EDIT_WIDTH + 2;
          let bottom = top + EDIT_HEIGHT + 2;
          let edit_outline = RECT { 
            left: left,
            top: top,
            right: right,
            bottom: bottom
          };
          *EDIT_OUTLINE.lock().unwrap() = edit_outline;
          let left = x - EDIT_WIDTH / 2 - 1;
          let top = (y - EDIT_HEIGHT / 2 + 1) - 19 - 1;
          let right = left + EDIT_WIDTH + 2;
          let bottom = top + EDIT2_HEIGHT + 2;
          let edit2_outline = RECT { 
            left: left,
            top: top,
            right: right,
            bottom: bottom
          };
          *EDIT2_OUTLINE.lock().unwrap() = edit2_outline;
        }
      });

      self.lbl.on().stn_clicked({
        let self2 = self.clone();
        move || {
          let lbl = &self2.lbl;
          let text = match lbl.text().unwrap().as_ref() {
            "Start" => { 
              *IS_TICKING.lock().unwrap() = true;
              let text = &self2.edt.text().unwrap().to_owned();
              let mut parts = text.split_whitespace();
              let num_str = parts.next().unwrap();
              let num_f32 = num_str.parse::<f32>().unwrap();
              let target_milliseconds = (num_f32 * 1000_f32) as i32;
              *TARGET_MILLISECONDS.lock().unwrap() = target_milliseconds;
              self2.wnd.hwnd().SetTimer(1, 10, None).unwrap();
              "Pause"
            },
            "Pause" => {
              *IS_TICKING.lock().unwrap() = false;
              "Resume"
            },
            "Resume" => {
              *IS_TICKING.lock().unwrap() = true;
              "Pause"
            },
            "Restart" => {
              *IS_TICKING.lock().unwrap() = true;
              *ELAPSED_MILLISECONDS.lock().unwrap() = 0;
              "Pause"
            },
            _ => unreachable!()
          };
          self2.lbl.set_text(text).unwrap();
        }
      });

      self.lbl.on().stn_dbl_clk({
        let self2 = self.clone();
        move || {
          let lbl = &self2.lbl;
          let text = match lbl.text().unwrap().as_ref() {
            "Start" => { 
              *IS_TICKING.lock().unwrap() = true;
              self2.wnd.hwnd().SetTimer(1, 10, None).unwrap();
              "Pause"
            },
            "Pause" => {
              *IS_TICKING.lock().unwrap() = false;
              "Resume"
            },
            "Resume" => {
              *IS_TICKING.lock().unwrap() = true;
              "Pause"
            },
            "Restart" => {
              *IS_TICKING.lock().unwrap() = true;
              *ELAPSED_MILLISECONDS.lock().unwrap() = 0;
              "Pause"
            },
            _ => unreachable!()
          };
        self2.lbl.set_text(text).unwrap();
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
          white_brush
        }
      });

      self.wnd.on().wm_paint({
        let self2 = self.clone();
        move || {
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

          let orange = COLORREF::new(0xff, 0x7f, 0x50);
          let orange_brush = HBRUSH::CreateSolidBrush(orange).unwrap();

          let (
            top_bar, 
            left_bar, 
            bottom_bar, 
            right_bar
          ) = calculate_progress_bars(rect, progress_width);

          hdc.FillRect(bottom_bar, orange_brush).unwrap();
          hdc.FillRect(top_bar, orange_brush).unwrap();
          hdc.FillRect(left_bar, orange_brush).unwrap();
          if progress_width >= rect.right - thickness { hdc.FillRect(right_bar, orange_brush).unwrap(); }
          
          let edit_outline = *EDIT_OUTLINE.lock().unwrap();
          let color = COLORREF::new(0xb5, 0xcf, 0xe7);
          let brush = HBRUSH::CreateSolidBrush(color).unwrap();
          hdc.FillRect(edit_outline, brush).unwrap();

          let edit2_outline = *EDIT2_OUTLINE.lock().unwrap();
          hdc.FillRect(edit2_outline, brush).unwrap();

          self2.wnd.hwnd().EndPaint(&ps);
        }
      });
    }
}

fn calculate_grey_bars(client_rect: RECT, progress_width: i32) -> (RECT, RECT, RECT, RECT) {
  let thickness = *THICKNESS.lock().unwrap();

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
    right: client_rect.right, 
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