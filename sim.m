
s = tf('s');

R1 = 10e3;
C1 = 22e-6;
R2 = 220e3;
C2 = 100e-9;

# serial RC
Z1 = R1 + 1/(s*C1);
# parallel RC
Z2 = 1/(s*C2 + 1/R2);

# AOP transfer function
Stage1 = (Z1 + Z2) / Z1;

bode(Stage1)
x_objs = findobj(gcf,'-property','XData');
for k=1:4
  set(x_objs(k), 'XData', get(x_objs(k),'XData')/(2*pi));
end
xlabel('Frequency (Hz)');
h1 = gcf;
ax = findall(h1,'type','axes');
set(ax(4),'title', 'Bode diagram of stage 1');
print -dpng -color -landscape bode_stage1.png
