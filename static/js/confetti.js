// Standalone HTML5 Canvas Confetti Animation
class ConfettiManager {
  constructor() {
    this.canvas = null;
    this.ctx = null;
    this.particles = [];
    this.animating = false;
  }

  ensureCanvas() {
    if (!this.canvas) {
      this.canvas = document.createElement("canvas");
      this.canvas.id = "confetti-canvas";
      document.body.appendChild(this.canvas);
      this.ctx = this.canvas.getContext("2d");
      this.resize();
      window.addEventListener("resize", () => this.resize());
    }
  }

  resize() {
    if (this.canvas) {
      this.canvas.width = window.innerWidth;
      this.canvas.height = window.innerHeight;
    }
  }

  burst(count = 120) {
    this.ensureCanvas();
    const colors = ["#e21b3c", "#1368ce", "#ffa602", "#26890c", "#ec4899", "#8b5cf6", "#f59e0b"];

    for (let i = 0; i < count; i++) {
      this.particles.push({
        x: window.innerWidth * 0.5,
        y: window.innerHeight * 0.6,
        vx: (Math.random() - 0.5) * 22,
        vy: -Math.random() * 18 - 8,
        size: Math.random() * 9 + 6,
        color: colors[Math.floor(Math.random() * colors.length)],
        rotation: Math.random() * 360,
        rotationSpeed: (Math.random() - 0.5) * 15,
        opacity: 1,
        life: 1,
        decay: Math.random() * 0.008 + 0.005,
      });
    }

    if (!this.animating) {
      this.animating = true;
      requestAnimationFrame(() => this.loop());
    }
  }

  loop() {
    if (!this.ctx) return;
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    for (let i = this.particles.length - 1; i >= 0; i--) {
      const p = this.particles[i];
      p.x += p.vx;
      p.y += p.vy;
      p.vy += 0.45; // gravity
      p.vx *= 0.98; // air drag
      p.rotation += p.rotationSpeed;
      p.life -= p.decay;

      if (p.life <= 0 || p.y > this.canvas.height + 50) {
        this.particles.splice(i, 1);
        continue;
      }

      this.ctx.save();
      this.ctx.translate(p.x, p.y);
      this.ctx.rotate((p.rotation * Math.PI) / 180);
      this.ctx.fillStyle = p.color;
      this.ctx.globalAlpha = Math.max(0, p.life);
      this.ctx.fillRect(-p.size / 2, -p.size / 2, p.size, p.size * 0.6);
      this.ctx.restore();
    }

    if (this.particles.length > 0) {
      requestAnimationFrame(() => this.loop());
    } else {
      this.animating = false;
      this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }
  }
}

window.confetti = new ConfettiManager();
