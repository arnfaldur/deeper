package main

import (
	"math"
	"math/cmplx"
)

type Entity struct {
	id        ID
	pos       complex128
	name      string
	collision bool
	solid     bool
	vel       complex128
	size      float64
	weight    float64
}

func NewTile(id ID, x float64, y float64) Entity {
	return Entity{id: id, pos: complex(x, y), collision: isSolid[id], solid: isSolid[id]}
}

type Character struct {
	Entity
	maxHealth, currHealth int
	attributes            [7]int
	damage                int
}

type Player struct {
	Character
}

type NPC struct {
	Character
	aggro bool
}

func (n *NPC) update(p *Player) {
}

func (v *Character) attackBy(a Character) {
	v.currHealth -= a.damage
}

func (e *Entity) entityCollision() {

}

func (e *Entity) tileCollide(theMap *Mapt) {
	px, py := parts(e.pos)

	pxf, pxr, pxc, pyf, pyr, pyc := int(px), int(px+0.5), int(math.Nextafter(px+1, math.Inf(-1))), int(py), int(py+0.5), int(math.Nextafter(py+1, math.Inf(-1)))
	any := false
	toWall := e.size / 2
	if isSolid[theMap[pyf][pxr].id] && toWall >= math.Abs(py-0.5-float64(pyf)) {
		e.vel = complex(real(e.vel), math.Max(0, imag(e.vel)))
		e.pos = complex(real(e.pos), float64(pyf)+0.5+toWall)
		any = true
	} else if isSolid[theMap[pyc][pxr].id] && toWall >= math.Abs(py+0.5-float64(pyc)) {
		e.vel = complex(real(e.vel), math.Min(0, imag(e.vel)))
		e.pos = complex(real(e.pos), float64(pyc)-0.5-toWall)
		any = true
	}
	if isSolid[theMap[pyr][pxf].id] && toWall >= math.Abs(px-0.5-float64(pxf)) {
		e.vel = complex(math.Max(0, real(e.vel)), imag(e.vel))
		e.pos = complex(float64(pxf)+0.5+toWall, imag(e.pos))
		any = true
	} else if isSolid[theMap[pyr][pxc].id] && toWall >= math.Abs(px+0.5-float64(pxc)) {
		e.vel = complex(math.Min(0, real(e.vel)), imag(e.vel))
		e.pos = complex(float64(pxc)-0.5-toWall, imag(e.pos))
		any = true
	}
	if !any {
		fs := [4]float64{py - 0.5, py + 0.5, px - 0.5, px + 0.5}
		is := [4]int{pyf, pyc, pxf, pxc}
		for y := 0; y < 2; y++ {
			for x := 2; x < 4; x++ {
				colDir := complex(fs[x]-float64(is[x]), fs[y]-float64(is[y]))
				if isSolid[theMap[is[y]][is[x]].id] && toWall > cmplx.Abs(colDir) {
					colDep := toWall - cmplx.Abs(colDir)
					e.vel += cmplx.Rect(colDep/2, cmplx.Phase(colDir))
					e.pos += cmplx.Rect(colDep, cmplx.Phase(colDir))
					//e.vel += cmul(colDir,colDep)
					//e.pos += cmul(colDir,colDep)
					break
				}
			}
		}
	}
}

func (c *Character) npcCollide(npcs *[]NPC) {
	for i, n := range *npcs {
		colDir := c.pos - n.pos
		colDep := (c.size+n.size)/2 - cmplx.Abs(colDir)
		if c.pos != n.pos && colDep > 0 {
			nudge := cmul(colDir, colDep)
			//nudge := cmplx.Rect(colDep/6, cmplx.Phase(colDir))
			//println(colDep)
			c.pos += nudge
			(*npcs)[i].pos -= nudge
			if c.id == PLAYER {
				(*npcs)[i].currHealth -= c.damage
			} else {
				c.vel = (c.vel+n.vel)/2 + nudge
				(*npcs)[i].vel = (c.vel+n.vel)/2 - nudge
			}
		}
	}
}

func (p *Player) update(theMap *Mapt, npcs *[]NPC, moveDirection complex128) {
	if cmplx.Abs(moveDirection) > 1 {
		moveDirection = cmplxNorm(moveDirection)
	}
	p.vel = approach(p.vel, moveDirection/5) * complex(timeDilation, 0)
	p.pos += p.vel
	for i := range *npcs {
		(*npcs)[i].pos += (*npcs)[i].vel
	}
	p.npcCollide(npcs)
	for i, e := range *npcs {
		if e.aggro {
			diff := p.pos - e.pos
			(*npcs)[i].vel = approach(e.vel, diff/complex(cmplx.Abs(diff), 0)/10) * complex(timeDilation, 0)
		}
		e.npcCollide(npcs)
	}
	p.tileCollide(theMap)
	for i := range *npcs {
		(*npcs)[i].tileCollide(theMap)
	}
}

func testEnemyNPC(pos complex128, id int) NPC {
	return NPC{Character{Entity: Entity{id: makeActorID(id), pos: pos, name: "TestEnemy", size: 0.8}, maxHealth: 10, currHealth: 10}, true}
}

func dummyNPC(x, y int) NPC {
	return NPC{Character{Entity: Entity{id: DUMMY, pos: complex(float64(x), float64(y)), name: "dummy"}, maxHealth: 10, currHealth: 10}, true}
}

func approach(vel, target complex128) complex128 {
	return (vel*4 + target) / 5
}
