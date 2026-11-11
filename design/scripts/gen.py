import os, sys
OUT = sys.argv[1]
TEAL="oklch(0.42 0.085 197)"; TEALD="oklch(0.34 0.08 202)"; CREAM="oklch(0.96 0.025 95)"
POP="oklch(0.86 0.16 92)"; POPD="oklch(0.72 0.15 82)"; INK="oklch(0.17 0.022 200)"
PINK="oklch(0.83 0.07 25)"; PAPER="oklch(0.986 0.006 190)"

def eyes(kind, dx=0, dy=0):
    if kind=="open":
        return f'<circle cx="{84+dx}" cy="{93+dy}" r="4.6" fill="{POP}"/><circle cx="{116+dx}" cy="{93+dy}" r="4.6" fill="{POP}"/>'
    if kind=="closed":
        return f'<path d="M77 95 Q84 100 91 95 M109 95 Q116 100 123 95" stroke="{POP}" stroke-width="2.8" fill="none" stroke-linecap="round"/>'
    if kind=="happy":
        return f'<path d="M77 97 Q84 88 91 97 M109 97 Q116 88 123 97" stroke="{POP}" stroke-width="3" fill="none" stroke-linecap="round"/>'

def mouse(eye="open", dx=0, dy=0, mouth="smile", paws="rest", cheeks=False, back="", front="", tilt=0):
    m = {"smile": f'<path d="M93 121 Q100 128 107 121" stroke="{INK}" stroke-width="2.4" fill="none" stroke-linecap="round"/>',
         "o": f'<ellipse cx="100" cy="123" rx="3" ry="3.6" fill="{INK}"/>',
         "chew": f'<path d="M94 122 Q97 125 100 122 Q103 125 106 122" stroke="{INK}" stroke-width="2.2" fill="none" stroke-linecap="round"/>',
         "big": f'<path d="M90 119 Q100 134 110 119 Z" fill="{INK}"/><path d="M95 125 Q100 129 105 125" fill="{PINK}"/>'}[mouth]
    p = {"rest": f'<ellipse cx="80" cy="142" rx="9" ry="7" fill="{CREAM}" stroke="{INK}" stroke-width="2.5"/><ellipse cx="120" cy="142" rx="9" ry="7" fill="{CREAM}" stroke="{INK}" stroke-width="2.5"/>',
         "up": f'<path d="M66 132 Q52 118 50 100 M134 132 Q148 118 150 100" stroke="{INK}" stroke-width="12" fill="none" stroke-linecap="round"/><path d="M66 132 Q52 118 50 100 M134 132 Q148 118 150 100" stroke="{CREAM}" stroke-width="7" fill="none" stroke-linecap="round"/>',
         "hold": f'<ellipse cx="88" cy="138" rx="8" ry="6.5" fill="{CREAM}" stroke="{INK}" stroke-width="2.5"/><ellipse cx="112" cy="138" rx="8" ry="6.5" fill="{CREAM}" stroke="{INK}" stroke-width="2.5"/>'}[paws]
    ck = f'<circle cx="72" cy="112" r="7" fill="{PINK}" opacity=".6"/><circle cx="128" cy="112" r="7" fill="{PINK}" opacity=".6"/>' if cheeks else ""
    holdfront = paws=="hold"
    return f'''{back}
<path d="M142 166 C188 172 198 128 176 112" stroke="{TEAL}" stroke-width="5" fill="none" stroke-linecap="round"/>
<ellipse cx="80" cy="182" rx="13" ry="6" fill="{CREAM}" stroke="{INK}" stroke-width="2.5"/>
<ellipse cx="120" cy="182" rx="13" ry="6" fill="{CREAM}" stroke="{INK}" stroke-width="2.5"/>
<ellipse cx="100" cy="148" rx="46" ry="36" fill="{CREAM}" stroke="{INK}" stroke-width="3"/>
{"" if holdfront else p}
<g transform="rotate({tilt} 100 95)">
<circle cx="64" cy="60" r="22" fill="{CREAM}" stroke="{INK}" stroke-width="3"/><circle cx="64" cy="60" r="12" fill="{PINK}"/>
<circle cx="136" cy="60" r="22" fill="{CREAM}" stroke="{INK}" stroke-width="3"/><circle cx="136" cy="60" r="12" fill="{PINK}"/>
<circle cx="100" cy="96" r="40" fill="{CREAM}" stroke="{INK}" stroke-width="3"/>
{ck}
<path d="M72 91 L61 87 M128 91 L139 87" stroke="{INK}" stroke-width="3" stroke-linecap="round"/>
<path d="M95 92 Q100 87 105 92" stroke="{INK}" stroke-width="3" fill="none" stroke-linecap="round"/>
<circle cx="84" cy="94" r="12.5" fill="{INK}"/><circle cx="116" cy="94" r="12.5" fill="{INK}"/>
{eyes(eye, dx, dy)}
<circle cx="100" cy="112" r="4.2" fill="{PINK}" stroke="{INK}" stroke-width="1.6"/>
<path d="M90 115 L64 110 M90 118 L64 121 M110 115 L136 110 M110 118 L136 121" stroke="{INK}" stroke-width="1.4" opacity=".55" stroke-linecap="round"/>
{m}
</g>
{p if holdfront else ""}
{front}'''

def cheese_wheel():
    return f'''<g><path d="M150 186 L214 186 L214 150 Z" fill="{POP}" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>
<circle cx="200" cy="176" r="5" fill="{POPD}"/><circle cx="186" cy="180" r="3" fill="{POPD}"/><circle cx="206" cy="163" r="3" fill="{POPD}"/></g>'''

def zzz():
    return f'<g fill="{INK}" font-family="system-ui" font-weight="800" opacity=".55"><text x="146" y="44" font-size="20">z</text><text x="162" y="28" font-size="14">z</text><text x="174" y="16" font-size="10">z</text></g>'

def held_cheese(uid):
    return f'''<mask id="b{uid}"><rect width="220" height="200" fill="white"/><circle cx="84" cy="128" r="9" fill="black"/><circle cx="94" cy="124" r="7" fill="black"/></mask>
<g mask="url(#b{uid})"><path d="M80 150 L128 150 L128 124 Z" fill="{POP}" stroke="{INK}" stroke-width="3" stroke-linejoin="round"/>
<circle cx="118" cy="143" r="3.5" fill="{POPD}"/><circle cx="123" cy="134" r="2" fill="{POPD}"/></g>
<rect x="70" y="160" width="4" height="4" fill="{POP}" transform="rotate(20 72 162)"/><rect x="132" y="168" width="3" height="3" fill="{POP}"/>'''

def party():
    sp = lambda x,y,s: f'<path d="M{x} {y-s} L{x+s*.3} {y-s*.3} L{x+s} {y} L{x+s*.3} {y+s*.3} L{x} {y+s} L{x-s*.3} {y+s*.3} L{x-s} {y} L{x-s*.3} {y-s*.3} Z" fill="{POP}"/>'
    crumbs = "".join(f'<rect x="{x}" y="{y}" width="4" height="4" fill="{POP}" transform="rotate({r} {x} {y})"/>' for x,y,r in [(56,188,15),(146,190,40),(160,184,10),(40,182,30),(170,192,60)])
    return sp(30,40,10)+sp(186,60,8)+sp(24,120,6)+sp(196,140,11)+crumbs

def plate():
    return f'''<ellipse cx="100" cy="134" rx="30" ry="7" fill="white" stroke="{INK}" stroke-width="2.5"/><ellipse cx="100" cy="133" rx="18" ry="3.5" fill="none" stroke="{INK}" stroke-width="1.2" opacity=".4"/>
<rect x="104" y="128" width="3" height="3" fill="{POP}"/>'''

def bubble(text, x, y, w, dark=False):
    bg, fg = (CREAM, TEALD) if dark else ("white", INK)
    return f'<g><rect x="{x}" y="{y}" width="{w}" height="30" rx="14" fill="{bg}" stroke="{INK}" stroke-width="2.5"/><text x="{x+w/2}" y="{y+20}" text-anchor="middle" font-family="Bricolage Grotesque, system-ui" font-weight="800" font-size="15" fill="{fg}">{text}</text></g>'

svg = lambda body, vb="0 0 220 200": f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">{body}</svg>'

moods = [
 ("sleepy", "Underused", "Asleep next to a wedge it'll never finish.", "“That's a lot of cheese for one mouse. Pro would do.”",
   mouse(eye="closed", mouth="o", tilt=-6, back=cheese_wheel()+zzz()), PAPER),
 ("nibbling", "On its way", "Eating through the wedge as you use Claude.", "“62% of the way to paid off.”",
   mouse(eye="open", dx=-2, dy=3, mouth="chew", paws="hold", cheeks=True, front=held_cheese("n")), PAPER),
 ("paid-off", "Break-even", "Pops onto the screen the moment the plan pays for itself.", "“Paid off! Everything from here is a bonus.”",
   mouse(eye="happy", mouth="big", paws="up", front=party()+bubble("paid off!", 62, 4, 96, dark=True)), TEAL),
 ("still-hungry", "Hitting limits", "Holds up an empty plate when you keep hitting rate limits.", "“Still hungry… Max 5x might be worth it.”",
   mouse(eye="open", dy=-4, mouth="o", paws="hold", front=plate()+bubble("more?", 140, 20, 66)), PAPER),
]

def wedge(level, uid):
    cut = 2 + 16*(1-level)
    bites = "" if level>=1 else f'<rect x="0" y="0" width="{cut}" height="20" fill="black"/>' + "".join(f'<circle cx="{cut}" cy="{y}" r="2.1" fill="black"/>' for y in (9.5,13,16.5))
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><mask id="w{uid}"><rect width="20" height="20" fill="white"/>
<circle cx="14" cy="13" r="1.6" fill="black"/><circle cx="16.3" cy="9.3" r="1" fill="black"/><circle cx="10.5" cy="14.6" r="1.1" fill="black"/>{bites}</mask>
<path d="M2 16.5 L18 16.5 L18 5.5 Z" fill="currentColor" stroke="currentColor" stroke-width="1" stroke-linejoin="round" mask="url(#w{uid})"/></svg>'''

def crumbs():
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><g fill="currentColor"><rect x="4" y="14" width="2.4" height="2.4" transform="rotate(20 5 15)"/><rect x="9" y="15" width="2" height="2"/><rect x="13.5" y="13.5" width="2.6" height="2.6" transform="rotate(40 14.5 14.5)"/></g></svg>'

def head_icon(uid, color=False):
    pupil = f'<circle cx="7.6" cy="11.6" r=".9" fill="{POP}"/><circle cx="12.4" cy="11.6" r=".9" fill="{POP}"/>' if color else ""
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><mask id="h{uid}"><rect width="20" height="20" fill="white"/>
<circle cx="7.6" cy="11.6" r="1.9" fill="black"/><circle cx="12.4" cy="11.6" r="1.9" fill="black"/></mask>
<g fill="currentColor" mask="url(#h{uid})"><circle cx="4.6" cy="6" r="3.4"/><circle cx="15.4" cy="6" r="3.4"/><circle cx="10" cy="11.5" r="6"/></g>{pupil}</svg>'''

def paid(color=False):
    chk = POP if color else "currentColor"
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><path d="M2 16.5 L18 16.5 L18 5.5 Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
<path d="M11.4 13 L13.1 14.6 L16.2 11.1" stroke="{chk}" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>'''

icons = [("100%", "", wedge(1,"a")), ("66%", "", wedge(.66,"b")), ("33%", "", wedge(.33,"c")), ("0%", "", crumbs()), ("paid off", "1.2×", paid())]

os.makedirs(OUT, exist_ok=True)
for slug, *_r, body, _bg in moods:
    open(os.path.join(OUT, f"nibble-{slug}.svg"), "w").write(svg(body))

cards = "".join(f'''<figure class="card{' drench' if bg==TEAL else ''}"><div class="art">{svg(body)}</div>
<figcaption><span class="label">{label}</span><p>{desc}</p><q>{quote}</q></figcaption></figure>''' for slug,label,desc,quote,body,bg in moods)

def bar(theme):
    items = "".join(f'<div class="ic"><span class="i">{s}</span>{"<b>"+x+"</b>" if x else ""}<small>{t}</small></div>' for t,x,s in icons)
    return f'<div class="bar {theme}"><span class="clock">Wed 11 Nov 14:02</span>{items}</div>'

html = f'''<!doctype html><html><head><meta charset="utf-8"><title>nibble moods</title><style>
body{{margin:0;font-family:"Public Sans",system-ui,sans-serif}} .wrap{{background:{PAPER};color:{INK};padding:22px;border-radius:var(--radius,14px)}}
h1{{font-family:"Bricolage Grotesque",system-ui;font-weight:800;font-size:44px;letter-spacing:-.035em;margin:0;color:{TEALD}}}
h1 em{{font-style:normal;color:{POPD}}}
.sub{{color:oklch(0.46 0.024 200);margin:6px 0 32px;font-size:17px}}
.grid{{display:grid;grid-template-columns:repeat(2,1fr);gap:18px}}
.card{{margin:0;background:white;border:1px solid oklch(0.88 0.012 195);border-radius:18px;padding:18px;display:flex;flex-direction:column}}
.card.drench{{background:linear-gradient(155deg,{TEAL},{TEALD});color:{CREAM};border-color:transparent}}
.art svg{{width:100%;height:auto}}
.label{{font-family:"JetBrains Mono",ui-monospace,monospace;font-size:12px;letter-spacing:.06em;text-transform:uppercase;opacity:.75}}
figcaption p{{margin:6px 0 8px;font-size:15px;line-height:1.5}}
q{{font-family:"Bricolage Grotesque",system-ui;font-weight:700;font-size:16px;quotes:none}}
h2{{font-family:"Bricolage Grotesque",system-ui;font-size:26px;margin:40px 0 6px;color:{TEALD}}}
.bar{{display:flex;flex-wrap:wrap;align-items:center;gap:16px;padding:6px 18px;border-radius:10px;margin-top:12px;font-size:13px}}
.bar.light{{background:oklch(0.94 0.005 250);color:#1a1a1a}} .bar.dark{{background:#232325;color:#f2f2f2}}
.clock{{order:99;margin-left:auto;opacity:.8}}
.ic{{display:flex;align-items:center;gap:8px}} .i svg{{width:18px;height:18px;display:block}}
.ic b{{font-size:13px}} .ic small{{opacity:.6;font-family:"JetBrains Mono",monospace;font-size:11px}}
.big{{display:flex;flex-wrap:wrap;gap:22px;margin-top:18px;color:{TEALD}}} .big svg{{width:72px;height:72px}}
</style></head><body><div class="wrap">
<h1>nibble<em>.</em></h1><p class="sub">Is your Claude plan paying for itself? A mouse eats through your plan, one wedge of cheese at a time.</p>
<div class="grid">{cards}</div>
<h2>Menu bar</h2><p class="sub" style="margin:0">Template icons at real size (18px). The wedge shrinks as you use Claude. Once the plan has paid off, it turns into a checked outline, and the multiplier counts up as you keep using Claude.</p>
{bar("light")}{bar("dark")}
<div class="big">{"".join(s for _,_,s in icons[:4])}{paid(color=True)}</div>
</div></body></html>'''
open(os.path.join(OUT, "nibble-moods.html"), "w").write(html)
os.makedirs(os.path.join(OUT, "menubar"), exist_ok=True)
for name, svgtxt in [("wedge-100", wedge(1,"a")), ("wedge-66", wedge(.66,"b")), ("wedge-33", wedge(.33,"c")), ("crumbs", crumbs()), ("paid-off", paid())]:
    open(os.path.join(OUT, "menubar", f"{name}.svg"), "w").write(svgtxt.replace("currentColor", "#000"))
print("ok")
