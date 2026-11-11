import sys
OUT=sys.argv[1]
POP="oklch(0.86 0.16 92)"; TEALD="oklch(0.34 0.08 202)"; INK="oklch(0.17 0.022 200)"; PAPER="oklch(0.986 0.006 190)"
S=lambda b:f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">{b}</svg>'
head_mask=lambda u,cx=10,cy=11.5,extra="":f'<mask id="{u}"><rect width="20" height="20" fill="white"/><circle cx="{cx-2.4}" cy="{cy+.1}" r="1.9" fill="black"/><circle cx="{cx+2.4}" cy="{cy+.1}" r="1.9" fill="black"/>{extra}</mask>'
def head(u,cx=10,cy=11.5,r=6,extra=""):
    return head_mask(u,cx,cy,extra)+f'<g fill="currentColor" mask="url(#{u})"><circle cx="{cx-5.4*r/6}" cy="{cy-5.5*r/6}" r="{3.4*r/6}"/><circle cx="{cx+5.4*r/6}" cy="{cy-5.5*r/6}" r="{3.4*r/6}"/><circle cx="{cx}" cy="{cy}" r="{r}"/></g>'
check=lambda x,y,s=1:f'<path d="M{x-2.2*s} {y} L{x-.6*s} {y+1.6*s} L{x+2.4*s} {y-1.6*s}" stroke="currentColor" stroke-width="{1.7*s}" fill="none" stroke-linecap="round" stroke-linejoin="round"/>'
opts=[
 ("A","Check badge","nibble's head with a ✓ badge: “done, paid”.",
  S(head("a",cx=8.4,cy=10.6,r=5.4,extra='<circle cx="15" cy="15" r="5.2" fill="black"/>')+'<mask id="ac"><rect width="20" height="20" fill="white"/><path d="M13.1 15.1 L14.5 16.5 L17 13.8" stroke="black" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/></mask><circle cx="15" cy="15" r="4" fill="currentColor" mask="url(#ac)"/>'),""),
 ("B","Checked wedge","Same cheese outline as the other states, now empty with a ✓ inside: “plan used up”.",
  S('<path d="M2 16.5 L18 16.5 L18 5.5 Z" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>'+check(13.4,13,.85)),""),
 ("C","Head + multiplier","Same head, with the ratio as text next to it. It counts up as you use more.",
  S(head("d")),"1.2×"),
 ("D","Check + multiplier","Option B's checked wedge plus the ratio: clear at a glance and gives the number.",
  S('<path d="M2 16.5 L18 16.5 L18 5.5 Z" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>'+check(13.4,13,.85)),"1.2×"),
]
crumbs=S('<g fill="currentColor"><rect x="4" y="14" width="2.4" height="2.4" transform="rotate(20 5 15)"/><rect x="9" y="15" width="2" height="2"/><rect x="13.5" y="13.5" width="2.6" height="2.6" transform="rotate(40 14.5 14.5)"/></g>')
def item(svg,txt): return f'<span class="it"><span class="i">{svg}</span>{f"<b>{txt}</b>" if txt else ""}</span>'
rows="".join(f'''<div class="row"><div class="meta"><span class="k">{k}</span><div><strong>{t}</strong><p>{d}</p></div></div>
<div class="bars"><div class="bar light">{item(s,x)}</div><div class="bar dark">{item(s,x)}</div><div class="zoom">{s}</div></div></div>''' for k,t,d,s,x in opts)
html=f'''<!doctype html><html><head><meta charset="utf-8"><style>
body{{margin:0;font-family:"Public Sans",system-ui,sans-serif}}
.wrap{{background:{PAPER};color:{INK};padding:20px;border-radius:var(--radius,14px)}}
h1{{font-family:"Bricolage Grotesque",system-ui;font-weight:800;font-size:26px;margin:0 0 4px;color:{TEALD}}}
.sub{{margin:0 0 14px;color:oklch(0.46 0.024 200);font-size:14px}}
.row{{display:flex;align-items:center;gap:16px;padding:12px 0;border-top:1px solid oklch(0.88 0.012 195)}}
.meta{{flex:1;display:flex;gap:12px;align-items:flex-start}} .meta p{{margin:2px 0 0;font-size:13.5px;line-height:1.45;color:oklch(0.4 0.024 200)}}
.k{{font-family:"JetBrains Mono",monospace;font-weight:700;background:{POP};color:{TEALD};border-radius:7px;padding:2px 8px}}
.bars{{display:flex;align-items:center;gap:10px}}
.bar{{height:26px;min-width:64px;display:flex;align-items:center;justify-content:center;padding:0 10px;border-radius:7px}}
.bar.light{{background:oklch(0.94 0.005 250);color:#1a1a1a}} .bar.dark{{background:#232325;color:#f2f2f2}}
.it{{display:flex;align-items:center;gap:4px;font-size:13px}} .i svg{{width:18px;height:18px;display:block}}
.zoom{{color:{TEALD}}} .zoom svg{{width:56px;height:56px;display:block}}
</style></head><body><div class="wrap"><h1>“paid off” icon options</h1>
<p class="sub">Shown at real menu bar size (light and dark) and zoomed in.</p>{rows}</div></body></html>'''
open(OUT,"w").write(html)
